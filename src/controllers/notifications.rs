use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use models::Notification;

use crate::Session;

#[instrument(skip(pool))]
pub async fn delete_notification(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let notification = Notification::get_by_id(id, &conn).await?;
	if session.data.profile_id != notification.notification.profile_id {
		return Err(Error::Forbidden);
	}

	Notification::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

#[instrument(skip(pool))]
pub async fn read_notification(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let notification = Notification::get_by_id(id, &conn).await?;
	if session.data.profile_id != notification.notification.profile_id {
		return Err(Error::Forbidden);
	}

	Notification::mark_read(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

#[instrument(skip(pool))]
pub async fn unread_notification(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let notification = Notification::get_by_id(id, &conn).await?;
	if session.data.profile_id != notification.notification.profile_id {
		return Err(Error::Forbidden);
	}

	Notification::mark_unread(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
