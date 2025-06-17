mod util;

use std::env;

use clap::{Error, Parser};
use common::DbConn;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel::RunQueryDsl;
use diesel::query_dsl::methods::SelectDsl;
use fake::faker::address::raw::{CityName, StateName, StreetName, ZipCode};
use fake::faker::chrono::raw::Time;
use fake::faker::company::raw::CompanyName;
use fake::faker::internet::raw::{FreeEmail, Password, Username};
use fake::faker::lorem::raw::Sentence;
use fake::locales::{DE_DE, EN, FR_FR};
use fake::{Dummy, Fake};
use models::{
	InsertableNewLocation,
	NewOpeningTime,
	NewProfileDirect,
	NewTranslation,
	ProfileState,
};
use rand::seq::IndexedRandom;
use rand::{Rng, rng};

use crate::util::{batch_insert, generate_unique_set};

#[derive(Parser, Debug)]
struct Opt {
	#[arg(long, short = 'p', default_value_t = 100_000)]
	profiles:      usize,
	#[arg(long, short = 'l', default_value_t = 1_000)]
	locations:     usize,
	#[arg(long, short = 't', default_value_t = 5_000)]
	opening_times: usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let cli = Opt::parse();
	let conn = get_conn().await;

	if cli.profiles > 0 {
		println!("Seeding {} profiles…", cli.profiles);
		let inserted = seed_profiles(&conn, cli.profiles).await?;
		println!("Inserted {inserted} unique profiles");
	}

	if cli.locations > 0 {
		println!("Seeding {} locations…", cli.locations);
		let inserted = seed_locations(&conn, cli.locations).await?;
		println!("Inserted {inserted} locations with translations");
	}

	if cli.opening_times > 0 {
		println!("Seeding {} opening times…", cli.opening_times);
		let inserted = seed_opening_times(&conn, cli.opening_times).await?;
		println!("Inserted {inserted} opening times for locations");
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
			NewOpeningTime {
				location_id:      *location_ids.choose(&mut rng).unwrap(),
				day:              RelevantDate
					.fake::<chrono::NaiveDateTime>()
					.date(),
				start_time:       Time(EN).fake(),
				end_time:         Time(EN).fake(),
				seat_count:       (10..100).fake_with_rng(&mut rng),
				reservable_from:  RelevantDate.fake(),
				reservable_until: RelevantDate.fake(),
				created_by:       *profile_ids.choose(&mut rng).unwrap(),
			}
		})
		.collect();

	batch_insert(conn, opening_times, 2 << 10, |conn, chunk| {
		use models::schema::opening_time::dsl::*;
		diesel::insert_into(opening_time).values(chunk).execute(conn)
	})
	.await
}
