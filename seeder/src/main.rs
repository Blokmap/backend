mod util;

use std::env;

use clap::{Error, Parser};
use common::DbConn;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use fake::faker::address::raw::{CityName, StateName, StreetName, ZipCode};
use fake::faker::company::raw::CompanyName;
use fake::faker::internet::raw::{FreeEmail, Password, Username};
use fake::faker::lorem::raw::Sentence;
use fake::locales::{DE_DE, EN, FR_FR};
use fake::{Dummy, Fake};
use models::{
	InsertableNewLocation,
	NewOpeningTime,
	NewProfileDirect,
	NewReservation,
	NewTranslation,
	ProfileState,
	ReservationIncludes,
};
use rand::seq::IndexedRandom;
use rand::{Rng, rng};

use crate::util::{batch_insert, generate_unique_set};

#[derive(Parser, Debug)]
struct Opt {
	#[arg(long, short = 'p')]
	profiles:              Option<usize>,
	#[arg(long, short = 'l')]
	locations:             Option<usize>,
	#[arg(long, short = 't')]
	opening_times:         Option<usize>,
	#[arg(long)]
	seed_reservations_for: Option<i32>,
	#[arg(long, default_value = "100")]
	reservation_count:     usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let cli = Opt::parse();
	let conn = get_conn().await;

	if let Some(profiles) = cli.profiles {
		println!("Seeding {} profiles…", profiles);
		let inserted = seed_profiles(&conn, profiles).await?;
		println!("Inserted {inserted} unique profiles");
	}

	if let Some(locations) = cli.locations {
		println!("Seeding {} locations…", locations);
		let inserted = seed_locations(&conn, locations).await?;
		println!("Inserted {inserted} locations with translations");
	}

	if let Some(opening_times) = cli.opening_times {
		println!("Seeding {} opening times…", opening_times);
		let inserted = seed_opening_times(&conn, opening_times).await?;
		println!("Inserted {inserted} opening times for locations");
	}

	if let Some(profile_id) = cli.seed_reservations_for {
		println!(
			"Seeding {} reservations for profile ID {}…",
			cli.reservation_count, profile_id
		);
		let inserted =
			seed_reservations(&conn, profile_id, cli.reservation_count).await?;
		println!(
			"Inserted {inserted} reservations for profile ID {profile_id}"
		);
	}

	Ok(())
}

/// Get a database connection from the pool
async fn get_conn() -> DbConn {
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");

	let manager = Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
	let pool = Pool::builder(manager).build().expect("Failed to create pool");

	pool.get().await.expect("Failed to get a database connection")
}

/// Seed profiles with unique usernames and emails
async fn seed_profiles(conn: &DbConn, count: usize) -> Result<usize, Error> {
	let usernames =
		generate_unique_set(count, || Username(EN).fake::<String>());
	let emails = generate_unique_set(count, || FreeEmail(EN).fake::<String>());

	let profiles: Vec<NewProfileDirect> = usernames
		.into_iter()
		.zip(emails)
		.map(|(username, email)| {
			let password_hash = Password(EN, 1..10).fake::<String>();
			NewProfileDirect {
				username,
				password_hash,
				email: Some(email),
				state: ProfileState::Active,
			}
		})
		.collect();

	batch_insert(conn, profiles, 8192, |conn, chunk| {
		use models::db::profile::dsl::*;
		diesel::insert_into(profile).values(chunk).execute(conn)
	})
	.await
}

async fn seed_translations(
	conn: &DbConn,
	count: usize,
	creator_ids: &[i32],
) -> Result<Vec<i32>, Error> {
	let mut rng = rng();

	let entries = (0..count)
		.map(|_| {
			let created_by = *creator_ids.choose(&mut rng).unwrap();

			NewTranslation {
				nl: Some(Sentence(EN, 1..3).fake()),
				en: Some(Sentence(EN, 1..3).fake()),
				fr: Some(Sentence(FR_FR, 1..3).fake()),
				de: Some(Sentence(DE_DE, 1..3).fake()),
				created_by,
			}
		})
		.collect();

	batch_insert(conn, entries, 2 << 10, |conn, chunk| {
		use models::db::translation::dsl::*;
		diesel::insert_into(translation).values(chunk).execute(conn)
	})
	.await?;

	let inserted_ids = conn
		.interact(move |c| {
			use models::db::translation::dsl::*;
			translation.select(id).load::<i32>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	Ok(inserted_ids)
}

/// Seed locations with random data and translations
async fn seed_locations(conn: &DbConn, count: usize) -> Result<usize, Error> {
	let profile_ids: Vec<i32> = conn
		.interact(|c| {
			use models::db::profile::dsl::*;
			profile.select(id).load::<i32>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	assert!(
		!profile_ids.is_empty(),
		"No profiles exist to assign as location creators"
	);

	let descriptions = seed_translations(conn, count, &profile_ids).await?;
	let excerpts = seed_translations(conn, count, &profile_ids).await?;

	let mut rng = rng();

	let locations: Vec<InsertableNewLocation> = (0..count)
		.map(|i| {
			let name = CompanyName(EN).fake::<String>();
			let description_id = descriptions[i % descriptions.len()];
			let excerpt_id = excerpts[i % excerpts.len()];
			let seat_count = (10..100).fake_with_rng(&mut rng);
			let is_reservable = rng.random_bool(0.4);
			let reservation_block_size = (15..120).fake_with_rng(&mut rng);
			let min_reservation_length = (1..4).fake_with_rng(&mut rng);
			let max_reservation_length = (2..24).fake_with_rng(&mut rng);
			let street = StreetName(EN).fake::<String>();
			let number = (1..200).fake_with_rng::<u32, _>(&mut rng).to_string();
			let zip = ZipCode(EN).fake::<String>();
			let city = CityName(EN).fake::<String>();
			let country = "BE".to_string();
			let province = StateName(EN).fake();
			let latitude = rng.random_range(49.5..=51.5);
			let longitude = rng.random_range(2.5..=6.4);
			let created_by = *profile_ids.choose(&mut rng).unwrap();

			InsertableNewLocation {
				name,
				authority_id: None,
				description_id,
				excerpt_id,
				seat_count,
				is_reservable,
				reservation_block_size,
				min_reservation_length,
				max_reservation_length,
				street,
				number,
				zip,
				city,
				country,
				province,
				latitude,
				longitude,
				created_by,
			}
		})
		.collect();

	batch_insert(conn, locations, 2 << 10, |conn, chunk| {
		use models::db::location::dsl::*;
		diesel::insert_into(location).values(chunk).execute(conn)
	})
	.await
}

async fn seed_reservations(
	conn: &DbConn,
	profile_id: i32,
	count: usize,
) -> Result<usize, Error> {
	// Get opening times with their associated location data
	let available_times: Vec<(i32, chrono::NaiveTime, chrono::NaiveTime, i32, Option<i32>, Option<i32>, chrono::NaiveDate)> = conn
		.interact(|c| {
			use models::db::opening_time::dsl::*;
			use models::db::location::dsl as loc_dsl;
			
			opening_time
				.inner_join(loc_dsl::location.on(location_id.eq(loc_dsl::id)))
				.select((
					id,
					start_time,
					end_time,
					loc_dsl::reservation_block_size,
					loc_dsl::min_reservation_length,
					loc_dsl::max_reservation_length,
					day,
				))
				.load::<(i32, chrono::NaiveTime, chrono::NaiveTime, i32, Option<i32>, Option<i32>, chrono::NaiveDate)>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	if available_times.is_empty() {
		return Ok(0);
	}

	// Get existing reservations for this user to avoid overlaps
	let existing_reservations: Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime)> = conn
		.interact(move |c| {
			use models::db::reservation::dsl::*;
			use models::db::opening_time::dsl as ot_dsl;
			use models::db::location::dsl as loc_dsl;
			
			reservation
				.inner_join(ot_dsl::opening_time.on(opening_time_id.eq(ot_dsl::id)))
				.inner_join(loc_dsl::location.on(ot_dsl::location_id.eq(loc_dsl::id)))
				.filter(profile_id.eq(profile_id))
				.select((
					ot_dsl::day,
					ot_dsl::start_time,
					base_block_index,
					block_count,
					loc_dsl::reservation_block_size,
				))
				.load::<(chrono::NaiveDate, chrono::NaiveTime, i32, i32, i32)>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.into_iter()
		.map(|(day, opening_start, base_idx, block_cnt, block_size)| {
			let start_offset = chrono::Duration::minutes((base_idx * block_size).into());
			let end_offset = chrono::Duration::minutes(((base_idx + block_cnt) * block_size).into());
			let reservation_start = day.and_time(opening_start + start_offset);
			let reservation_end = day.and_time(opening_start + end_offset);
			(reservation_start, reservation_end)
		})
		.collect();

	let mut rng = rng();
	let mut successful_reservations = Vec::new();
	let existing_count = existing_reservations.len();
	let mut created_reservations: Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime)> = existing_reservations;

	// Try to create reservations, checking for overlaps
	for _ in 0..count {
		let mut attempts = 0;
		let max_attempts = 50; // Prevent infinite loops
		
		while attempts < max_attempts {
			let (opening_time_id, start_time, end_time, block_size_minutes, min_length_opt, max_length_opt, day) = 
				*available_times.choose(&mut rng).unwrap();
			
			// Calculate total available blocks in the opening time
			let total_duration_minutes = (end_time - start_time).num_minutes();
			let total_blocks = (total_duration_minutes / i64::from(block_size_minutes)) as i32;
			
			// Ensure reservation is at least 1 hour (60 minutes)
			let min_blocks_for_1_hour = (60.0 / f64::from(block_size_minutes)).ceil() as i32;
			let min_length = min_length_opt.unwrap_or(1);
			let max_length = max_length_opt.unwrap_or(total_blocks);
			
			let min_blocks = std::cmp::max(min_blocks_for_1_hour, min_length);
			let max_blocks = std::cmp::min(total_blocks, max_length);
			
			// If we can't fit a 1-hour reservation, try another opening time
			if min_blocks > max_blocks || max_blocks <= 0 {
				attempts += 1;
				continue;
			}
			
			// Generate random block count between min and max
			let reservation_blocks = rng.random_range(min_blocks..=max_blocks);
			
			// Generate random starting position that allows the reservation to fit
			let max_base_index = std::cmp::max(0, total_blocks - reservation_blocks);
			let base_block_index = if max_base_index > 0 {
				rng.random_range(0..=max_base_index)
			} else {
				0
			};

			// Calculate the actual time span of this reservation
			let start_offset = chrono::Duration::minutes((base_block_index * block_size_minutes).into());
			let end_offset = chrono::Duration::minutes(((base_block_index + reservation_blocks) * block_size_minutes).into());
			let reservation_start = day.and_time(start_time + start_offset);
			let reservation_end = day.and_time(start_time + end_offset);

			// Check for overlaps with existing reservations
			let has_overlap = created_reservations.iter().any(|(existing_start, existing_end)| {
				// Two time ranges overlap if one starts before the other ends
				reservation_start < *existing_end && reservation_end > *existing_start
			});

			if !has_overlap {
				// No overlap found, create the reservation
				let new_reservation = NewReservation {
					profile_id,
					opening_time_id,
					base_block_index,
					block_count: reservation_blocks,
				};
				
				successful_reservations.push(new_reservation);
				created_reservations.push((reservation_start, reservation_end));
				break; // Successfully created a reservation, move to next one
			}
			
			attempts += 1;
		}
		
		// If we couldn't find a non-overlapping slot after max_attempts, we'll just skip this reservation
	}

	// Insert all successful reservations
	for reservation in successful_reservations {
		let _ = reservation
			.insert(ReservationIncludes::default(), conn)
			.await
			.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;
	}

	Ok(created_reservations.len() - existing_count)
}

/// Faked DateTime that is bounded to the current week +- 1 week
struct RelevantDate;

impl Dummy<RelevantDate> for chrono::NaiveDateTime {
	fn dummy_with_rng<R: Rng + ?Sized>(_: &RelevantDate, rng: &mut R) -> Self {
		let now = chrono::Utc::now().date_naive();
		let week = now.week(chrono::Weekday::Mon);
		let start = (week.checked_first_day().unwrap() - chrono::Days::new(7))
			.and_time(chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap());
		let end = (week.checked_last_day().unwrap() + chrono::Days::new(7))
			.and_time(chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap());

		let start = start.and_utc().timestamp();
		let end = end.and_utc().timestamp();

		let random_secs = rng.random_range(start..end);

		let datetime =
			chrono::DateTime::from_timestamp(random_secs, 0).unwrap();
		datetime.naive_utc()
	}
}

async fn seed_opening_times(
	conn: &DbConn,
	count: usize,
) -> Result<usize, Error> {
	let profile_ids: Vec<i32> = conn
		.interact(|c| {
			use models::db::profile::dsl::*;
			profile.select(id).load::<i32>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	assert!(
		!profile_ids.is_empty(),
		"No profiles exist to assign as opening time creators"
	);

	let location_ids: Vec<i32> = conn
		.interact(|c| {
			use models::db::location::dsl::*;
			location.select(id).get_results(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	assert!(
		!location_ids.is_empty(),
		"No locations exist to create opening times"
	);

	let mut rng = rng();

	let opening_times: Vec<NewOpeningTime> = (0..count)
		.map(|_| {
			// Generate a start time that allows for at least 15 minutes and up
			// to 6 hours Start time between 6:00 and 17:59 (to allow for at
			// least 6 hours until 23:59)
			let start_hour = rng.random_range(6..18);
			let start_minute = rng.random_range(0..60);
			let start_time =
				chrono::NaiveTime::from_hms_opt(start_hour, start_minute, 0)
					.unwrap();

			// Calculate maximum possible duration to not exceed 23:59:59
			let max_end_time =
				chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap();
			let max_duration_minutes =
				(max_end_time - start_time).num_minutes();

			// Generate a duration between 15 minutes and min(6 hours, time
			// until end of day)
			let max_duration = std::cmp::min(360, max_duration_minutes); // 360 min = 6 hours
			let duration_minutes = rng.random_range(15..=max_duration);
			let end_time =
				start_time + chrono::Duration::minutes(duration_minutes);

			NewOpeningTime {
				location_id: *location_ids.choose(&mut rng).unwrap(),
				day: RelevantDate.fake::<chrono::NaiveDateTime>().date(),
				start_time,
				end_time,
				seat_count: (10..100).fake_with_rng(&mut rng),
				reservable_from: RelevantDate.fake(),
				reservable_until: RelevantDate.fake(),
				created_by: *profile_ids.choose(&mut rng).unwrap(),
			}
		})
		.collect();

	batch_insert(conn, opening_times, 2 << 10, |conn, chunk| {
		use models::db::opening_time::dsl::*;
		diesel::insert_into(opening_time).values(chunk).execute(conn)
	})
	.await
}
