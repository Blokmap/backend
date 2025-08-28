use ::profile::Profile;
use chrono::NaiveDateTime;
use common::{DbConn, Error};
use db::{
	authority,
	authority_member,
	creator,
	image,
	institution,
	profile,
	updater,
};
use diesel::pg::Pg;
use diesel::prelude::*;
use primitives::{PrimitiveAuthority, PrimitiveProfile};
use serde::{Deserialize, Serialize};

use crate::{Authority, AuthorityIncludes};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = authority_member)]
#[diesel(check_for_backend(Pg))]
pub struct AuthorityProfile {
	pub id:           i32,
	pub authority_id: i32,
	pub profile_id:   i32,
	pub added_at:     NaiveDateTime,
	pub added_by:     Option<i32>,
	pub updated_at:   NaiveDateTime,
	pub updated_by:   Option<i32>,
}

impl Authority {
	/// Get all [members](SimpleProfile) of this [`Authority`]
	#[instrument(skip(conn))]
	pub async fn get_members(
		auth_id: i32,
		conn: &DbConn,
	) -> Result<Vec<PrimitiveProfile>, Error> {
		let members = conn
			.interact(move |conn| {
				authority_member::table
					.filter(authority_member::authority_id.eq(auth_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(authority_member::profile_id)),
					)
					.select(PrimitiveProfile::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(members)
	}

	/// Delete a member from this authority
	#[instrument(skip(conn))]
	pub async fn delete_member(
		auth_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::authority_member::dsl::*;

			diesel::delete(
				authority_member.filter(
					authority_id.eq(auth_id).and(profile_id.eq(prof_id)),
				),
			)
			.execute(conn)
		})
		.await??;

		info!("deleted profile {prof_id} from authority {auth_id}");

		Ok(())
	}

	/// Get all [`Authorities`](Authority) for a given profile
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::joined_query(includes);

		let authorities = conn
			.interact(move |conn| {
				use self::authority_member::dsl::*;

				authority_member
					.filter(profile_id.eq(p_id))
					.inner_join(query.on(authority_id.eq(authority::id)))
					.select((
						PrimitiveAuthority::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
						institution::all_columns.nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(authorities)
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = authority_member)]
#[diesel(check_for_backend(Pg))]
pub struct NewAuthorityProfile {
	pub authority_id: i32,
	pub profile_id:   i32,
	pub added_by:     i32,
}

impl NewAuthorityProfile {
	/// Insert this [`NewAuthorityProfile`]
	#[instrument(skip(conn))]
	pub async fn insert(self, conn: &DbConn) -> Result<Profile, Error> {
		conn.interact(move |conn| {
			use self::authority_member::dsl::*;

			diesel::insert_into(authority_member).values(self).execute(conn)
		})
		.await??;

		let profile = conn
			.interact(move |conn| {
				authority_member::table
					.filter(
						authority_member::authority_id
							.eq(self.authority_id)
							.and(
								authority_member::profile_id
									.eq(self.profile_id),
							),
					)
					.inner_join(
						profile::table
							.on(profile::id.eq(authority_member::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select(Profile::as_select())
					.get_result(conn)
			})
			.await??;

		info!(
			"added profile {} to authority {}",
			self.profile_id, self.authority_id
		);

		Ok(profile)
	}
}
