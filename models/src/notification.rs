use chrono::{NaiveDateTime, Utc};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::PrimitiveTranslation;
use crate::schema::{notification, translation};

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = notification)]
#[diesel(check_for_backend(Pg))]
pub struct Notification {
	pub notification: PrimitiveNotification,
	pub body:         PrimitiveTranslation,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = notification)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveNotification {
	pub id:         i32,
	pub profile_id: i32,
	pub body_id:    i32,
	pub created_at: NaiveDateTime,
	pub read_at:    Option<NaiveDateTime>,
}

impl Notification {
	/// Get a [`Notification`] given its ID
	#[instrument(skip(conn))]
	pub async fn get_by_id(n_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let (notification, body) = conn
			.interact(move |conn| {
				use crate::schema::notification::dsl::*;

				notification
					.find(n_id)
					.inner_join(translation::table)
					.select((
						PrimitiveNotification::as_select(),
						PrimitiveTranslation::as_select(),
					))
					.get_result(conn)
			})
			.await??;

		let notification = Self { notification, body };

		Ok(notification)
	}

	/// Get all [`Notification`]s for a profile with the given ID
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let notifs = conn
			.interact(move |conn| {
				use crate::schema::notification::dsl::*;

				notification
					.filter(profile_id.eq(p_id))
					.inner_join(translation::table)
					.select((
						PrimitiveNotification::as_select(),
						PrimitiveTranslation::as_select(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(notification, body)| Self { notification, body })
			.collect();

		Ok(notifs)
	}

	/// Mark this [`Notification`] as read
	#[instrument(skip(conn))]
	pub async fn mark_read(n_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::schema::notification::dsl::*;

			diesel::update(notification.find(n_id))
				.set(read_at.eq(Utc::now().naive_utc()))
				.execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Mark this [`Notification`] as unread
	#[instrument(skip(conn))]
	pub async fn mark_unread(n_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::schema::notification::dsl::*;

			diesel::update(notification.find(n_id))
				.set(read_at.eq(None::<NaiveDateTime>))
				.execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Delete this [`Notification`]
	#[instrument(skip(conn))]
	pub async fn delete_by_id(n_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::schema::notification::dsl::*;

			diesel::delete(notification.find(n_id)).execute(conn)
		})
		.await??;

		Ok(())
	}
}
