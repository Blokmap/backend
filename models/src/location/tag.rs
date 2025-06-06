use std::collections::HashMap;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::{
	ExpressionMethods,
	JoinOnDsl,
	NullableExpressionMethods,
	QueryDsl,
	Queryable,
	RunQueryDsl,
	Selectable,
	SelectableHelper,
};

use crate::schema::{profile, tag, translation};
use crate::{Profile, Translation};

#[derive(Clone, Debug, diesel::Identifiable, Queryable, Selectable)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
pub struct DbTag {
	pub id:                  i32,
	pub name_translation_id: i32,
	pub created_at:          NaiveDateTime,
	pub updated_at:          NaiveDateTime,
	pub created_by:          Option<i32>,
	pub updated_by:          Option<i32>,
}

pub struct Tag {
	pub tag:        DbTag,
	pub name:       Translation,
	pub created_by: Option<Option<Profile>>,
	pub updated_by: Option<Option<Profile>>,
}

impl Tag {
	/// Get all [`Tag`]s from the database, optionally including related
	/// profiles.
	pub async fn get_all(
		conn: &DbConn,
		include: &[&str],
	) -> Result<Vec<Tag>, Error> {
		let inc_created = include.contains(&"created_by");
		let inc_updated = include.contains(&"updated_by");

		let tags = conn
			.interact(move |c| {
				let mut q = tag::table
					.inner_join(
						translation::table
							.on(tag::name_translation_id.eq(translation::id)),
					)
					.into_boxed::<Pg>();

				if inc_created {
					q = q.left_join(
						profile::table
							.on(tag::created_by.eq(profile::id.nullable())),
					);
				}

				if inc_updated {
					q = q.left_join(
						profile::table
							.on(tag::updated_by.eq(profile::id.nullable())),
					);
				}

				q.select((
					DbTag::as_select(),
					Translation::as_select(),
					if inc_created {
						Some(Profile::as_select().nullable())
					} else {
						None
					},
					if inc_updated {
						Some(Profile::as_select().nullable())
					} else {
						None
					},
				))
				.load(c)
			})
			.await??;

		Ok(tags
			.into_iter()
			.map(|(tag, name, created_by, updated_by)| {
				Tag {
					tag,
					name,
					created_by: if inc_created {
						Some(created_by)
					} else {
						None
					},
					updated_by: if inc_updated {
						Some(updated_by)
					} else {
						None
					},
				}
			})
			.collect())
	}
}
