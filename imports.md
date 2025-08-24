primitive
 - authority
 - institution
 - location
 - image
 - opening_time
 - profile
 - reservation
 - review
 - tag
 - translation

full
 - authority:    [P.institution, P.profile]
 - institution:  [P.profile, P.translation, N.translation]
 - location:     [image, N.image, N.translation, P.authority, P.opening_time, P.profile, P.translation, tag]
 - image:        []
 - opening_time: [P.profile ]
 - profile:      [image, N.image, reservation]
 - reservation:  [P.location, P.opening_time, P.profile]
 - review:       [location, P.profile]
 - tag:          [N.translation, P.profile, P.translation, U.translation]
 - translation:  [P.profile ]


```rs
pub mod primitive {
	#[derive(Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize)]
	#[diesel(table_name = review)]
	#[diesel(check_for_backend(Pg))]
	pub struct PrimitiveReview {
		pub id:          i32,
		pub profile_id:  i32,
		pub location_id: i32,
		pub rating:      i32,
		pub body:        Option<String>,
		pub created_at:  NaiveDateTime,
		pub updated_at:  NaiveDateTime,
	}
}

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
pub struct ReviewIncludes {
	#[serde(default)]
	pub location: bool,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct ReviewParts {
	#[diesel(embed)]
	pub primitive:  primitive::PrimitiveReview,
	#[diesel(embed)]
	pub created_by: PrimitiveProfile,
	#[diesel(embed)]
	pub location:   Option<PrimitiveLocation>,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct Review {
	pub primitive:  primitive::PrimitiveReview,
	pub created_by: PrimitiveProfile,
	pub location:   Option<Option<PrimitiveLocation>>,
}

impl JoinParts for ReviewParts {
	type Target = Review;
	type Includes = ReviewIncludes;

	fn join(self, includes: &Self::Includes) -> Self::Target {
		Review {
			primitive:  self.primitive,
			created_by: self.created_by,
			location:   if includes.location { Some(self.location) } else { None },
		}
	}
}

impl Review {
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: &ReviewIncludes) -> _ {
		let loc_includes: bool = includes.location;

		review
			.join(profile.on(profile::id.eq(review.profile_id)))
			.left_join(location.on(loc_includes.into_sql::<Bool>().and(
				location::id.eq(review::location_id)
			)))
	}

	pub async fn get_by_id(r_id: i32, includes: &ReviewIncludes, conn: &DbConn) -> Result<Self, Error> {
		let parts = conn
			.interact(move |conn| {
				Self::query(includes)
					.filter(review::id.eq(r_id))
					.select(ReviewParts::as_select())
					.get_result(conn)

			})
			.await??;

		let review = parts.join(includes);

		Ok(review)
	}
}
```
