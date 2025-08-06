mod util;

use std::env;

use clap::{Error, Parser};
use common::DbConn;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::RunQueryDsl;
use diesel::query_dsl::methods::SelectDsl;
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
		use models::schema::profile::dsl::*;
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
		use models::schema::translation::dsl::*;
		diesel::insert_into(translation).values(chunk).execute(conn)
	})
	.await?;

	let inserted_ids = conn
		.interact(move |c| {
			use models::schema::translation::dsl::*;
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
			use models::schema::profile::dsl::*;
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
		use models::schema::location::dsl::*;
		diesel::insert_into(location).values(chunk).execute(conn)
	})
	.await
}

async fn seed_reservations(
	conn: &DbConn,
	profile_id: i32,
	count: usize,
) -> Result<usize, Error> {
	let available_times: Vec<(i32, Option<i32>)> = conn
		.interact(|c| {
			use models::schema::opening_time::dsl::*;
			opening_time.select((id, seat_count)).load::<(i32, Option<i32>)>(c)
		})
		.await
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?
		.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;

	if available_times.is_empty() {
		return Ok(0);
	}

	let mut rng = rng();
	let reservations: Vec<NewReservation> = (0..count)
		.map(|_| {
			let (t_id, _seats) = *available_times.choose(&mut rng).unwrap();
			NewReservation {
				profile_id,
				opening_time_id: t_id,
				base_block_index: rng.random_range(0..8),
				block_count: rng.random_range(3..=10),
			}
		})
		.collect();

	for reservation in reservations {
		let _ = reservation
			.insert(ReservationIncludes::default(), conn)
			.await
			.map_err(|e| Error::raw(clap::error::ErrorKind::Io, e))?;
	}

	Ok(count)
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
			use models::schema::profile::dsl::*;
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
			use models::schema::location::dsl::*;
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
		use models::schema::opening_time::dsl::*;
		diesel::insert_into(opening_time).values(chunk).execute(conn)
	})
	.await
}
