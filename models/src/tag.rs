use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;

use crate::schema::{profile, tag, translation};
use crate::{Profile, Translation};

#[derive(Clone, Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
pub struct DbTag {
	pub id:                  i32,
	pub name_translation_id: i32,
	pub created_at:          NaiveDateTime,
	pub created_by:          Option<i32>,
	pub updated_at:          NaiveDateTime,
	pub updated_by:          Option<i32>,
}

pub struct Tag {
	pub tag:        DbTag,
	pub name:       Translation,
	pub created_by: Option<Profile>,
	pub updated_by: Option<Profile>,
}

impl Tag {
	/// Get all [`Tag`]s from the database, optionally including related
	/// profiles.
	pub async fn get_all(
		conn: &DbConn,
		include: &[&str],
	) -> Result<Vec<Tag>, Error> {
		diesel::alias!(
			profile as creater: CreaterAlias,
			profile as updater: UpdaterAlias,
		);

		let inc_created = include.contains(&"created_by");
		let inc_updated = include.contains(&"updated_by");

		let tags = conn
			.interact(move |c| {
				let q = tag::table
					.inner_join(
						translation::table
							.on(tag::name_translation_id.eq(translation::id)),
					)
					.left_outer_join(
						creater.on(inc_created.into_sql::<Bool>().and(
							tag::created_by
								.eq(creater.field(profile::id).nullable()),
						)),
					)
					.left_outer_join(
						updater.on(inc_updated.into_sql::<Bool>().and(
							tag::updated_by
								.eq(updater.field(profile::id).nullable()),
						)),
					);

				q.select((
					DbTag::as_select(),
					Translation::as_select(),
					creater.fields(profile::all_columns).nullable(),
					updater.fields(profile::all_columns).nullable(),
				))
				.load(c)
			})
			.await??;

		Ok(tags
			.into_iter()
			.map(|(tag, name, created_by, updated_by)| {
				Tag { tag, name, created_by, updated_by }
			})
			.collect())
	}
}
