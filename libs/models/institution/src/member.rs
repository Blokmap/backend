use std::collections::HashMap;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
use db::{creator, image, institution, institution_profile, profile, updater};
use diesel::pg::Pg;
use diesel::prelude::*;
use primitives::{
	PrimitiveImage,
	PrimitiveInstitution,
	PrimitiveProfile,
	PrimitiveTranslation,
};
use serde::{Deserialize, Serialize};

use crate::{Institution, InstitutionIncludes};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = institution_profile)]
#[diesel(primary_key(institution_id, profile_id))]
#[diesel(check_for_backend(Pg))]
pub struct InstitutionProfile {
	pub institution_id: i32,
	pub profile_id:     i32,
	pub added_at:       NaiveDateTime,
	pub added_by:       Option<i32>,
	pub updated_at:     NaiveDateTime,
	pub updated_by:     Option<i32>,
	pub permissions:    i64,
}

bitflags! {
	/// Possible permissions for a member of an [`Institution`]
	#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
	pub struct InstitutionPermissions: i64 {
		/// Admin privileges, member can do everything
		const Administrator = 1 << 0;
	}
}

impl InstitutionPermissions {
	#[must_use]
	pub fn names() -> HashMap<&'static str, i64> {
		Self::all().iter_names().map(|(n, v)| (n, v.bits())).collect()
	}
}

impl Institution {
	/// Get all [members](SimpleProfile) of this [`Institution`] alongside
	/// their permissions
	#[instrument(skip(conn))]
	pub async fn get_members_with_permissions(
		inst_id: i32,
		conn: &DbConn,
	) -> Result<
		Vec<(PrimitiveProfile, Option<PrimitiveImage>, InstitutionPermissions)>,
		Error,
	> {
		let members = conn
			.interact(move |conn| {
				use self::institution_profile::dsl::*;

				institution_profile
					.filter(institution_id.eq(inst_id))
					.inner_join(profile::table.on(profile::id.eq(profile_id)))
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						permissions,
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(prof, img, perm): (_, _, i64)| {
				let perm = InstitutionPermissions::from_bits_truncate(perm);
				(prof, img, perm)
			})
			.collect();

		Ok(members)
	}

	/// Get the permissions for a single member
	#[instrument(skip(conn))]
	pub async fn get_member_permissions(
		inst_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<InstitutionPermissions, Error> {
		let permissions = conn
			.interact(move |conn| {
				use self::institution_profile::dsl::*;

				institution_profile
					.find((inst_id, prof_id))
					.select(permissions)
					.get_result(conn)
			})
			.await??;

		Ok(InstitutionPermissions::from_bits_truncate(permissions))
	}

	/// Delete a member from this institution
	#[instrument(skip(conn))]
	pub async fn delete_member(
		inst_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::institution_profile::dsl::*;

			diesel::delete(institution_profile.find((inst_id, prof_id)))
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
				use self::institution_profile::dsl::*;

				institution_profile
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
#[diesel(table_name = institution_profile)]
#[diesel(check_for_backend(Pg))]
pub struct NewInstitutionProfile {
	pub institution_id: i32,
	pub profile_id:     i32,
	pub added_by:       i32,
	pub permissions:    i64,
}

#[rustfmt::skip]
impl NewInstitutionProfile {
	/// Insert this [`NewInstitutionProfile`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		conn: &DbConn,
	) -> Result<(PrimitiveProfile, Option<PrimitiveImage>, InstitutionPermissions), Error>
	{
		conn.interact(move |conn| {
			use self::institution_profile::dsl::*;

			diesel::insert_into(institution_profile).values(self).execute(conn)
		})
		.await??;

		let (profile, img, permissions): (_, _, i64) = conn
			.interact(move |conn| {
				use self::institution_profile::dsl::*;

				institution_profile
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
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions =
			InstitutionPermissions::from_bits_truncate(permissions);

		info!(
			"added profile {} to institution {}",
			self.profile_id, self.institution_id
		);

		Ok((profile, img, permissions))
	}
}

#[derive(AsChangeset, Clone, Copy, Debug, Deserialize, Serialize)]
#[diesel(table_name = institution_profile)]
#[diesel(check_for_backend(Pg))]
pub struct InstitutionProfileUpdate {
	pub updated_by:  i32,
	pub permissions: i64,
}

#[rustfmt::skip]
impl InstitutionProfileUpdate {
	/// Apply this update to the [`Institution`] with the given id
	pub async fn apply_to(
		self,
		inst_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(PrimitiveProfile, Option<PrimitiveImage>, InstitutionPermissions), Error>
	{
		conn.interact(move |conn| {
			use self::institution_profile::dsl::*;

			diesel::update(institution_profile.find((inst_id, prof_id)))
				.set(self)
				.execute(conn)
		})
		.await??;

		let (profile, img, permissions): (_, _, i64) = conn
			.interact(move |conn| {
				institution_profile::table
					.find((inst_id, prof_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(institution_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						institution_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions =
			InstitutionPermissions::from_bits_truncate(permissions);

		info!(
			"set permissions for profile {} to {} in institution {}",
			prof_id, self.permissions, inst_id
		);

		Ok((profile, img, permissions))
	}
}
