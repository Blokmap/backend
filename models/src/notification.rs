use chrono::{NaiveDateTime, Utc};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::PrimitiveTranslation;
use crate::schema::{body, notification, title, translation};

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = notification)]
#[diesel(check_for_backend(Pg))]
pub struct Notification {
	pub notification: PrimitiveNotification,
	pub title:        PrimitiveTranslation,
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
	pub title_id:   i32,
	pub body_id:    i32,
	pub created_at: NaiveDateTime,
	pub read_at:    Option<NaiveDateTime>,
}

impl Notification {
	/// Get a [`Notification`] given its ID
	#[instrument(skip(conn))]
	pub async fn get_by_id(n_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let (notification, t, b) = conn
			.interact(move |conn| {
				use crate::schema::notification::dsl::*;

				notification
					.find(n_id)
					.inner_join(title.on(
						title.field(translation::id).eq(title_id)
					))
					.inner_join(body.on(
						body.field(translation::id).eq(body_id)
					))
					.select((
						PrimitiveNotification::as_select(),
						title.fields(
							<
								PrimitiveTranslation as Selectable<Pg>
							>::construct_selection()
						),
						body.fields(
							<
								PrimitiveTranslation as Selectable<Pg>
							>::construct_selection()
						),
					))
					.get_result(conn)
			})
			.await??;

		let notification = Self { notification, title: t, body: b };

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
					.inner_join(title.on(
						title.field(translation::id).eq(title_id)
					))
					.inner_join(body.on(
						body.field(translation::id).eq(body_id)
					))
					.select((
						PrimitiveNotification::as_select(),
						title.fields(
							<
								PrimitiveTranslation as Selectable<Pg>
							>::construct_selection()
						),
						body.fields(
							<
								PrimitiveTranslation as Selectable<Pg>
							>::construct_selection()
						),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(notification, t, b)| {
				Self { notification, title: t, body: b }
			})
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
