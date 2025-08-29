use ::profile::Profile;
use common::{DbConn, Error};
use db::{image, location_member, profile};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Location;

impl Location {
	/// Get all the members of this location
	#[instrument(skip(conn))]
	pub async fn get_members(
		l_id: i32,
		conn: &DbConn,
	) -> Result<Vec<Profile>, Error> {
		let members = conn
			.interact(move |conn| {
				location_member::table
					.filter(location_member::location_id.eq(l_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(location_member::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select(Profile::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(members)
	}

	/// Delete a member from this location
	#[instrument(skip(conn))]
	pub async fn delete_member(
		loc_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::location_member::dsl::*;

			diesel::delete(
				location_member
					.filter(location_id.eq(loc_id).and(profile_id.eq(prof_id))),
			)
			.execute(conn)
		})
		.await??;

		info!("deleted profile {prof_id} from location {loc_id}");

		Ok(())
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_member)]
#[diesel(check_for_backend(Pg))]
pub struct NewLocationMember {
	pub location_id: i32,
	pub profile_id:  i32,
	pub role_id:     Option<i32>,
	pub added_by:    i32,
}

impl NewLocationMember {
	/// Insert this [`NewLocationMember`]
	#[instrument(skip(conn))]
	pub async fn insert(self, conn: &DbConn) -> Result<Profile, Error> {
		conn.interact(move |conn| {
			use self::location_member::dsl::*;

			diesel::insert_into(location_member).values(self).execute(conn)
		})
		.await??;

		let profile = Profile::get(self.profile_id, conn).await?;

		info!(
			"added profile {} to location {}",
			self.profile_id, self.location_id
		);

		Ok(profile)
	}
}

#[derive(AsChangeset, Clone, Copy, Debug, Deserialize)]
#[diesel(table_name = location_member)]
#[diesel(check_for_backend(Pg))]
pub struct LocationMemberUpdate {
	pub role_id:    Option<i32>,
	pub updated_by: i32,
}

impl LocationMemberUpdate {
	/// Update this [`LocationMember`] in the database.
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		loc_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<Profile, Error> {
		conn.interact(move |conn| {
			use self::location_member::dsl::*;

			diesel::update(
				location_member
					.filter(location_id.eq(loc_id).and(profile_id.eq(prof_id))),
			)
			.set(self)
			.execute(conn)
		})
		.await??;

		let profile = Profile::get(prof_id, conn).await?;

		Ok(profile)
	}
}
