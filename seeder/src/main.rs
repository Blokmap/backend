mod util;

use std::env;

use clap::{Error, Parser};
use common::DbConn;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::RunQueryDsl;
use diesel::prelude::*;
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
	RESERVATION_BLOCK_SIZE_MINUTES,
};
use rand::seq::IndexedRandom;
use rand::{Rng, rng};

use crate::util::{batch_insert_optimized, generate_unique_set};

#[derive(Parser, Debug)]
struct Opt {
	#[arg(long, short = 'p')]
	profiles:              Option<usize>,
	#[arg(long, short = 'l')]
	locations:             Option<usize>,
	#[arg(long, short = 't')]
	opening_times:         Option<usize>,
	#[arg(long, short = 'r')]
	reservations:          Option<usize>,
	#[arg(long)]
	seed_reservations_for: Option<i32>,
	#[arg(long, default_value = "100")]
	reservation_count:     usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let cli = Opt::parse();
	let conn = get_conn().await;

	let start_time = std::time::Instant::now();

	if let Some(profiles) = cli.profiles {
		println!("Seeding {} profiles…", profiles);
		let profile_start = std::time::Instant::now();
		let inserted = seed_profiles(&conn, profiles).await?;
		println!(
			"✅ Inserted {} unique profiles in {:.2}s",
			inserted,
			profile_start.elapsed().as_secs_f64()
		);
	}

	if let Some(locations) = cli.locations {
		println!("Seeding {} locations…", locations);
		let location_start = std::time::Instant::now();
		let inserted = seed_locations(&conn, locations).await?;
		println!(
			"✅ Inserted {} locations with translations in {:.2}s",
			inserted,
			location_start.elapsed().as_secs_f64()
		);
	}

	if let Some(opening_times) = cli.opening_times {
		println!("Seeding {} opening times…", opening_times);
		let ot_start = std::time::Instant::now();
		let inserted = seed_opening_times(&conn, opening_times).await?;
		println!(
			"✅ Inserted {} opening times for locations in {:.2}s",
			inserted,
			ot_start.elapsed().as_secs_f64()
		);
	}

	if let Some(reservations) = cli.reservations {
		println!("Seeding {} reservations across all profiles…", reservations);
		let res_start = std::time::Instant::now();
		let inserted = seed_random_reservations(&conn, reservations).await?;
		println!(
			"✅ Inserted {} reservations in {:.2}s",
			inserted,
			res_start.elapsed().as_secs_f64()
		);
	}

	if let Some(profile_id) = cli.seed_reservations_for {
		println!(
			"Seeding {} reservations for profile ID {}…",
			cli.reservation_count, profile_id
		);
		let profile_res_start = std::time::Instant::now();
		let inserted = seed_reservations_for_profile(
			&conn,
			profile_id,
			cli.reservation_count,
		)
		.await?;
		println!(
			"✅ Inserted {} reservations for profile ID {} in {:.2}s",
			inserted,
			profile_id,
			profile_res_start.elapsed().as_secs_f64()
		);
	}

	println!(
		"All seeding completed in {:.2}s",
		start_time.elapsed().as_secs_f64()
	);
	Ok(())
}

/// Get a database connection from the pool
async fn get_conn() -> DbConn {
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");

	let manager = Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);

	// Pool configuration for bulk operations
	let pool = Pool::builder(manager)
		.max_size(16)
		.create_timeout(Some(std::time::Duration::from_secs(30)))
		.wait_timeout(Some(std::time::Duration::from_secs(30)))
		.recycle_timeout(Some(std::time::Duration::from_secs(30)))
		.runtime(deadpool_diesel::Runtime::Tokio1)
		.build()
		.expect("Failed to create pool");

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

	batch_insert_optimized(conn, profiles, 4, |conn, chunk| {
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

	batch_insert_optimized(conn, entries, 5, |conn, chunk| {
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

	println!("Creating translations for locations...");
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

	batch_insert_optimized(conn, locations, 18, |conn, chunk| {
		use models::db::location::dsl::*;
		diesel::insert_into(location).values(chunk).execute(conn)
	})
	.await
}

/// Seed reservations scattered across all profiles and locations - Optimized
/// for bulk generation
async fn seed_random_reservations(
	conn: &DbConn,
	count: usize,
) -> Result<usize, Error> {
	// Get all profiles
	let profile_ids: Vec<i32> = conn
		.interact(|c| {
			use models::db::profile::dsl::*;
			profile.select(id).load::<i32>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	if profile_ids.is_empty() {
		return Ok(0);
	}

	// Get available opening times with location data
	let available_times = get_available_opening_times(conn).await?;
	if available_times.is_empty() {
		return Ok(0);
	}

	let mut rng = rng();
	let mut reservations = Vec::with_capacity(count);

	// Generate all reservations at once for better performance
	for _ in 0..count {
		// Randomly pick a profile and opening time
		let profile_id = *profile_ids.choose(&mut rng).unwrap();
		let (opening_time_id, start_time, end_time, max_length_opt, day) =
			*available_times.choose(&mut rng).unwrap();

		if let Some(reservation) = create_valid_reservation(
			profile_id,
			opening_time_id,
			start_time,
			end_time,
			max_length_opt,
			day,
			&mut rng,
		) {
			reservations.push(reservation);
		}
	}

	if reservations.is_empty() {
		return Ok(0);
	}

	batch_insert_optimized(conn, reservations, 4, |conn, chunk| {
		use models::db::reservation::dsl::*;
		diesel::insert_into(reservation).values(chunk).execute(conn)
	})
	.await
}

/// Seed reservations for a specific profile, ensuring no time overlaps
async fn seed_reservations_for_profile(
	conn: &DbConn,
	profile_id: i32,
	count: usize,
) -> Result<usize, Error> {
	let available_times = get_available_opening_times(conn).await?;
	if available_times.is_empty() {
		return Ok(0);
	}

	let existing_reservations =
		get_existing_reservations_for_profile(conn, profile_id).await?;

	let mut rng = rng();
	let mut successful_reservations = Vec::with_capacity(count);
	let mut created_reservations = existing_reservations;

	let mut sorted_times = available_times;
	sorted_times
		.sort_by_key(|&(opening_time_id, _, _, _, day)| (opening_time_id, day));

	let max_attempts_per_reservation = 20;

	for i in 0..count {
		let mut attempts = 0;
		let mut found_valid = false;

		while attempts < max_attempts_per_reservation && !found_valid {
			// Try locations in a somewhat ordered fashion for better
			// performance
			let time_index = (i * 7 + attempts) % sorted_times.len(); // Pseudo-random but deterministic
			let (opening_time_id, start_time, end_time, max_length_opt, day) =
				sorted_times[time_index];

			if let Some(reservation) = create_valid_reservation(
				profile_id,
				opening_time_id,
				start_time,
				end_time,
				max_length_opt,
				day,
				&mut rng,
			) {
				// Calculate reservation time span
				let (reservation_start, reservation_end) =
					calculate_reservation_time_span(
						&reservation,
						day,
						start_time,
					);

				// Overlap check - check only recent reservations
				// first (most likely to conflict)
				let has_overlap =
					created_reservations.iter().rev().take(50).any(
						|(existing_start, existing_end)| {
							reservation_start < *existing_end
								&& reservation_end > *existing_start
						},
					) || created_reservations
						.iter()
						.take(created_reservations.len().saturating_sub(50))
						.any(|(existing_start, existing_end)| {
							reservation_start < *existing_end
								&& reservation_end > *existing_start
						});

				if !has_overlap {
					successful_reservations.push(reservation);
					created_reservations
						.push((reservation_start, reservation_end));
					found_valid = true;
				}
			}

			attempts += 1;
		}
	}

	if successful_reservations.is_empty() {
		return Ok(0);
	}

	let inserted = batch_insert_optimized(
		conn,
		successful_reservations,
		4,
		|conn, chunk| {
			use models::db::reservation::dsl::*;
			diesel::insert_into(reservation).values(chunk).execute(conn)
		},
	)
	.await?;

	Ok(inserted)
}

/// Helper function to get available opening times with location data
async fn get_available_opening_times(
	conn: &DbConn,
) -> Result<
	Vec<(
		i32,
		chrono::NaiveTime,
		chrono::NaiveTime,
		Option<i32>,
		chrono::NaiveDate,
	)>,
	Error,
> {
	conn.interact(|c| {
		use models::db::location::dsl as loc_dsl;
		use models::db::opening_time::dsl::*;

		opening_time
			.inner_join(loc_dsl::location.on(location_id.eq(loc_dsl::id)))
			.select((
				id,
				start_time,
				end_time,
				loc_dsl::max_reservation_length,
				day,
			))
			.load::<(
				i32,
				chrono::NaiveTime,
				chrono::NaiveTime,
				Option<i32>,
				chrono::NaiveDate,
			)>(c)
	})
	.await
	.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
	.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))
}

async fn get_existing_reservations_for_profile(
	conn: &DbConn,
	user_profile_id: i32,
) -> Result<Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime)>, Error> {
	let reservations = conn
		.interact(move |c| {
			use models::db::location::dsl as loc_dsl;
			use models::db::opening_time::dsl as ot_dsl;
			use models::db::reservation::dsl::*;

			reservation
				.inner_join(
					ot_dsl::opening_time.on(opening_time_id.eq(ot_dsl::id)),
				)
				.inner_join(
					loc_dsl::location.on(ot_dsl::location_id.eq(loc_dsl::id)),
				)
				.filter(profile_id.eq(user_profile_id))
				.order_by(ot_dsl::day.desc())
				.select((
					ot_dsl::day,
					ot_dsl::start_time,
					base_block_index,
					block_count,
				))
				.load::<(chrono::NaiveDate, chrono::NaiveTime, i32, i32)>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	let time_spans: Vec<(chrono::NaiveDateTime, chrono::NaiveDateTime)> =
		reservations
			.into_iter()
			.map(|(day, opening_start, base_idx, block_cnt)| {
				let start_offset = chrono::Duration::minutes(
					(base_idx * RESERVATION_BLOCK_SIZE_MINUTES).into(),
				);
				let end_offset = chrono::Duration::minutes(
					((base_idx + block_cnt) * RESERVATION_BLOCK_SIZE_MINUTES)
						.into(),
				);
				let reservation_start =
					day.and_time(opening_start + start_offset);
				let reservation_end = day.and_time(opening_start + end_offset);
				(reservation_start, reservation_end)
			})
			.collect();

	Ok(time_spans)
}

/// Helper function to create a valid reservation given constraints
fn create_valid_reservation(
	profile_id: i32,
	opening_time_id: i32,
	start_time: chrono::NaiveTime,
	end_time: chrono::NaiveTime,
	max_length_opt: Option<i32>,
	_day: chrono::NaiveDate,
	rng: &mut impl Rng,
) -> Option<NewReservation> {
	// Calculate total available blocks in the opening time
	let total_duration_minutes = (end_time - start_time).num_minutes();
	let total_blocks = (total_duration_minutes
		/ i64::from(RESERVATION_BLOCK_SIZE_MINUTES)) as i32;

	// Ensure reservation is at least 1 hour (60 minutes)
	let min_blocks_for_1_hour =
		(60.0 / f64::from(RESERVATION_BLOCK_SIZE_MINUTES)).ceil() as i32;
	let min_length = 1;
	let max_length = max_length_opt.unwrap_or(total_blocks);

	let min_blocks = std::cmp::max(min_blocks_for_1_hour, min_length);
	let max_blocks = std::cmp::min(total_blocks, max_length);

	// If we can't fit a 1-hour reservation, return None
	if min_blocks > max_blocks || max_blocks <= 0 {
		return None;
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

	Some(NewReservation {
		profile_id,
		opening_time_id,
		base_block_index,
		block_count: reservation_blocks,
	})
}

/// Helper function to calculate the actual time span of a reservation
fn calculate_reservation_time_span(
	reservation: &NewReservation,
	day: chrono::NaiveDate,
	opening_start_time: chrono::NaiveTime,
) -> (chrono::NaiveDateTime, chrono::NaiveDateTime) {
	let start_offset = chrono::Duration::minutes(
		(reservation.base_block_index * RESERVATION_BLOCK_SIZE_MINUTES).into(),
	);
	let end_offset = chrono::Duration::minutes(
		((reservation.base_block_index + reservation.block_count)
			* RESERVATION_BLOCK_SIZE_MINUTES)
			.into(),
	);
	let reservation_start = day.and_time(opening_start_time + start_offset);
	let reservation_end = day.and_time(opening_start_time + end_offset);
	(reservation_start, reservation_end)
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

	// NewOpeningTime has 7 parameters
	batch_insert_optimized(conn, opening_times, 7, |conn, chunk| {
		use models::db::opening_time::dsl::*;
		diesel::insert_into(opening_time).values(chunk).execute(conn)
	})
	.await
}
