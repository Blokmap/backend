//! Controllers for [`Profile`]s

use axum::extract::State;
use axum::{Extension, Json};

use crate::models::{Profile, ProfileId};
use crate::{DbPool, Error};

pub(crate) async fn get_all_profiles(
	State(pool): State<DbPool>,
) -> Result<Json<Vec<Profile>>, Error> {
	let conn = pool.get().await?;
	let profiles = Profile::get_all(&conn).await?;

	Ok(Json(profiles))
}

pub(crate) async fn get_current_profile(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
) -> Result<Json<Profile>, Error> {
	let conn = pool.get().await?;
	let profile = Profile::get(*profile_id, &conn).await?;

	Ok(Json(profile))
}
