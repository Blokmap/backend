use ::profile::Profile;
use chrono::NaiveDateTime;
use common::{DbConn, Error};
use db::{creator, image, institution, institution_member, profile, updater};
use diesel::pg::Pg;
use diesel::prelude::*;
use primitives::{PrimitiveInstitution, PrimitiveTranslation};
use serde::{Deserialize, Serialize};

use crate::{Institution, InstitutionIncludes};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = institution_member)]
#[diesel(check_for_backend(Pg))]
pub struct InstitutionMember {
	pub id:             i32,
	pub institution_id: i32,
	pub profile_id:     i32,
	pub added_at:       NaiveDateTime,
	pub added_by:       Option<i32>,
	pub updated_at:     NaiveDateTime,
	pub updated_by:     Option<i32>,
}

impl Institution {
	/// Get all [members](SimpleProfile) of this [`Institution`]
	#[instrument(skip(conn))]
	pub async fn get_members(
		inst_id: i32,
		conn: &DbConn,
	) -> Result<Vec<Profile>, Error> {
		let members = conn
			.interact(move |conn| {
				use self::institution_member::dsl::*;

				institution_member
					.filter(institution_id.eq(inst_id))
					.inner_join(profile::table.on(profile::id.eq(profile_id)))
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

	/// Delete a member from this institution
	#[instrument(skip(conn))]
	pub async fn delete_member(
		inst_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::institution_member::dsl::*;

			diesel::delete(
				institution_member.filter(
					institution_id.eq(inst_id).and(profile_id.eq(prof_id)),
				),
			)
			.execute(conn)
		})
		.await??;

		info!("deleted profile {prof_id} from institution {inst_id}");

		Ok(())
	}

	/// Get all [`Institutions`](Institution) for a given profile
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		includes: InstitutionIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::joined_query(includes);

		let institutions = conn
			.interact(move |conn| {
				use self::institution_member::dsl::*;

				institution_member
					.filter(profile_id.eq(p_id))
					.inner_join(query.on(institution_id.eq(institution::id)))
					.select((
						PrimitiveInstitution::as_select(),
						PrimitiveTranslation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(institutions)
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = institution_member)]
#[diesel(check_for_backend(Pg))]
pub struct NewInstitutionProfile {
	pub institution_id: i32,
	pub profile_id:     i32,
	pub added_by:       i32,
}

#[rustfmt::skip]
impl NewInstitutionProfile {
	/// Insert this [`NewInstitutionProfile`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		conn: &DbConn,
	) -> Result<Profile, Error>
	{
		conn.interact(move |conn| {
			use self::institution_member::dsl::*;

			diesel::insert_into(institution_member).values(self).execute(conn)
		})
		.await??;

		let profile = conn
			.interact(move |conn| {
				use self::institution_member::dsl::*;

				institution_member
					.filter(
						institution_id
							.eq(self.institution_id)
							.and(profile_id.eq(self.profile_id)),
					)
					.inner_join(profile::table.on(profile::id.eq(profile_id)))
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
			"added profile {} to institution {}",
			self.profile_id, self.institution_id
		);

		Ok(profile)
	}
}
