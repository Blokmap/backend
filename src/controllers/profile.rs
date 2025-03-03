use axum::Json;
use axum::extract::State;

use crate::DbPool;
use crate::error::Error;
use crate::models::Profile;

pub async fn get_all_profiles(
	State(pool): State<DbPool>,
) -> Result<Json<Vec<Profile>>, Error> {
	let conn = pool.get().await?;

	let profiles = Profile::get_all(conn).await?;

	Ok(Json(profiles))
}
