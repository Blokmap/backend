use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{profile, review};
use crate::{
	FullLocationData,
	Location,
	LocationIncludes,
	PrimitiveProfile,
	QUERY_HARD_LIMIT,
	manual_pagination,
};

pub type JoinedReviewData = (PrimitiveReview, PrimitiveProfile);

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct Review {
	pub review:     PrimitiveReview,
	pub created_by: PrimitiveProfile,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveReview {
	pub id:          i32,
	pub location_id: i32,
	pub rating:      i32,
	pub body:        Option<String>,
	pub created_at:  NaiveDateTime,
	pub updated_at:  NaiveDateTime,
}

impl Review {
	/// Build a query with all required (dynamic) joins to select a full
	/// review data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn joined_query() -> _ {
		review::table
			.inner_join(profile::table.on(profile::id.eq(review::profile_id)))
	}

	/// Construct a full [`Review`] struct from the data returned by a
	/// joined query
	fn from_joined(data: JoinedReviewData) -> Self {
		Self { review: data.0, created_by: data.1 }
	}

	/// Get all [`Review`]s for a location with the given ID
	#[instrument(skip(conn))]
	pub async fn for_location(
		l_id: i32,
		limit: usize,
		offset: usize,
		conn: &DbConn,
	) -> Result<(usize, bool, Vec<Self>), Error> {
		let query = Self::joined_query();

		let reviews = conn
			.interact(move |conn| {
				query
					.filter(review::location_id.eq(l_id))
					.select((
						PrimitiveReview::as_select(),
						PrimitiveProfile::as_select(),
					))
					.limit(QUERY_HARD_LIMIT)
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(Self::from_joined)
			.collect();

		manual_pagination(reviews, limit, offset)
	}

	/// Get all [`Review`]s for a profile with the given ID
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		conn: &DbConn,
	) -> Result<Vec<(Self, FullLocationData)>, Error> {
		let query = Self::joined_query();

		let (loc_ids, reviews): (Vec<i32>, Vec<Self>) = conn
			.interact(move |conn| {
				query
					.filter(review::profile_id.eq(p_id))
					.select((
						PrimitiveReview::as_select(),
						PrimitiveProfile::as_select(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data: JoinedReviewData| {
				(data.0.location_id, Self::from_joined(data))
			})
			.collect();

		let locations =
			Location::get_by_ids(loc_ids, LocationIncludes::default(), conn)
				.await?;

		let reviews = reviews
			.into_iter()
			.map(|r| {
				let loc = locations
					.iter()
					.find(|l| l.0.location.id == r.review.location_id)
					.unwrap()
					.to_owned();

				(r, loc)
			})
			.collect();

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
		let (review, created_by) = conn
			.interact(move |conn| {
				conn.transaction(|conn| {
					use crate::db::review::dsl::*;

					let r_id: i32 = diesel::insert_into(review)
						.values(self)
						.returning(id)
						.get_result(conn)?;

					review
						.find(r_id)
						.inner_join(
							profile::table.on(profile::id.eq(profile_id)),
						)
						.select((
							PrimitiveReview::as_select(),
							PrimitiveProfile::as_select(),
						))
						.get_result(conn)
				})
			})
			.await??;

		let review = Review { review, created_by };

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
		let (review, created_by) = conn
			.interact(move |conn| {
				conn.transaction(|conn| {
					use crate::db::review::dsl::*;

					let r_id: i32 = diesel::update(review)
						.set(self)
						.returning(id)
						.get_result(conn)?;

					review
						.find(r_id)
						.inner_join(
							profile::table.on(profile::id.eq(profile_id)),
						)
						.select((
							PrimitiveReview::as_select(),
							PrimitiveProfile::as_select(),
						))
						.get_result(conn)
				})
			})
			.await??;

		let review = Review { review, created_by };

		Ok(review)
	}
}
