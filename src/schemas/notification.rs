use chrono::NaiveDateTime;
use models::{Notification, PrimitiveTranslation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponse {
	pub id:         i32,
	pub body:       PrimitiveTranslation,
	pub created_at: NaiveDateTime,
	pub read_at:    Option<NaiveDateTime>,
}

impl From<Notification> for NotificationResponse {
	fn from(value: Notification) -> Self {
		Self {
			id:         value.notification.id,
			body:       value.body,
			created_at: value.notification.created_at,
			read_at:    value.notification.read_at,
		}
	}
}
