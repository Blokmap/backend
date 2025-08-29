#[macro_use]
extern crate tracing;

use std::default::Default;

use base::{
	PaginatedData,
	PaginationConfig,
	QUERY_HARD_LIMIT,
	manual_pagination,
};
use common::{DbConn, Error};
use db::{location, profile, review};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{PrimitiveLocation, PrimitiveProfile, PrimitiveReview};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct ReviewIncludes {
	#[serde(default)]
	pub location: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct Review {
	#[diesel(embed)]
	pub primitive:  PrimitiveReview,
	#[diesel(embed)]
	pub created_by: PrimitiveProfile,
	#[diesel(embed)]
	pub location:   Option<PrimitiveLocation>,
}

impl Review {
	/// Build a query with all required (dynamic) joins to select a full
	/// review data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: ReviewIncludes) -> _ {
		let inc_location: bool = includes.location;

		review::table
			.inner_join(profile::table.on(profile::id.eq(review::profile_id)))
			.left_join(
				location::table.on(inc_location
					.into_sql::<Bool>()
					.and(location::id.eq(review::location_id))),
			)
	}

	/// Get all [`Review`]s for a location with the given ID
	#[instrument(skip(conn))]
	pub async fn for_location(
		l_id: i32,
		includes: ReviewIncludes,
		p_cfg: PaginationConfig,
		conn: &DbConn,
	) -> Result<PaginatedData<Vec<Self>>, Error> {
		let reviews = conn
			.interact(move |conn| {
				Self::query(includes)
					.filter(review::location_id.eq(l_id))
					.select(Self::as_select())
					.limit(QUERY_HARD_LIMIT)
					.get_results(conn)
			})
			.await??;

		manual_pagination(reviews, p_cfg)
	}

	/// Get all [`Review`]s for a profile with the given ID
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		includes: ReviewIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let reviews = conn
			.interact(move |conn| {
				Self::query(includes)
					.filter(review::profile_id.eq(p_id))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(reviews)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct NewReview {
	pub profile_id:  i32,
	pub location_id: i32,
	pub rating:      i32,
	pub body:        Option<String>,
}

impl NewReview {
	/// Insert this [`NewReview`]
	#[instrument(skip(conn))]
	pub async fn insert(self, conn: &DbConn) -> Result<Review, Error> {
		let review = conn
			.interact(move |conn| {
				conn.transaction(|conn| {
					use self::review::dsl::*;

					let r_id: i32 = diesel::insert_into(review)
						.values(self)
						.returning(id)
						.get_result(conn)?;

					Review::query(ReviewIncludes::default())
						.filter(id.eq(r_id))
						.select(Review::as_select())
						.get_result(conn)
				})
			})
			.await??;

		Ok(review)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct ReviewUpdate {
	pub rating: Option<i32>,
	pub body:   Option<String>,
}

impl ReviewUpdate {
	/// Apply this update to the [`Review`] with the given id
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		r_id: i32,
		conn: &DbConn,
	) -> Result<Review, Error> {
		let review = conn
			.interact(move |conn| {
				conn.transaction(|conn| {
					use self::review::dsl::*;

					let r_id: i32 = diesel::update(review)
						.set(self)
						.returning(id)
						.get_result(conn)?;

					Review::query(ReviewIncludes::default())
						.filter(id.eq(r_id))
						.select(Review::as_select())
						.get_result(conn)
				})
			})
			.await??;

		Ok(review)
	}
}
