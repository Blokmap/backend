use askama::Template;
use common::Error;
use models::Profile;
// use multi_template::MultiTemplate;
use serde::{Deserialize, Serialize};

use crate::mailer::Mailer;

mod macro_test {
	use multi_template::MultiTemplate;

	#[derive(MultiTemplate)]
	enum Notification {
		#[multi_template(
			name = "confirm_email",
			variants(email, markdown, text)
		)]
		ConfirmEmail { confirmation_url: String },
		#[multi_template(name = "reset_password", variants(email))]
		ResetPassword { username: String, reset_url: String },
	}
}

/// #[derive(MultiTemplate)]
/// enum Notification {
/// 	#[multi_template(name = "confirm_email", variants("email", "markdown",
/// "text"))] 	ConfirmEmail { confirmation_url: String }
/// 	#[multi_template(name = "reset_password", variants("email"))]
/// 	ResetPassword { username: String, reset_url: String }
/// }
///
/// ==============
///
/// impl Notification {
/// 	fn fire_for<T>(self, profile: Profile) -> Result<Box<dyn
/// SendableNotifcation>, Error> { 		match self {
/// 			Self::ConfirmEmail { confirmation_url } => {
/// 				let email = ConfirmEmailTemplate::Email { confirmation_url }.render()?;
/// 				let markdown = ConfirmEmailTemplate::Markdown { confirmation_url
/// }.render()?; 				let text = ConfirmEmailTemplate::Text { confirmation_url
/// }.render()?;
///
/// 				Ok(Box::new(ConfirmEmail { email, markdown, text }))
/// 			},
/// 			Self::ResetPassword { reset_url } => {
/// 				let email = ResetPasswordTemplate::Email { reset_url }.render()?;
///
/// 				Ok(Box::new(ResetPassword { email }))
/// 			},
/// 		}
/// 	}
/// }
///
/// struct ConfirmEmailMultiTemplate {
/// 	email: String,
/// 	markdown: String,
/// 	text: String,
/// }
///
/// impl SendableNotification for ConfirmEmailMultiTemplate {
/// 	fn send_for(self, profile: Profile) -> Result<(), Error> {
///
/// 	}
/// }
///
/// #[derive(Template)]
/// enum ConfirmEmailTemplate {
/// 	#[template(path = "confirm_email/email.html")]
/// 	Email { confirmation_url: String }
/// 	#[template(path = "confirm_email/markdown.md")]
/// 	Markdown { confirmation_url: String }
/// 	#[template(path = "confirm_email/text.txt")]
/// 	Text { confirmation_url: String }
/// }
///
/// struct ResetPasswordMultiTemplate {
/// 	email: String,
/// }
///
/// #[derive(Template)]
/// enum ResetPasswordTemplate {
/// 	#[template(path = "confirm_email/email.html")]
/// 	Email { username: String, reset_url: String }
/// }

#[derive(Clone, Debug, Deserialize, Serialize, Template)]
pub enum Notification {
	#[template(path = "confirm_email/email.html")]
	ConfirmEmail { confirmation_url: String },
	#[template(path = "reset_password/email.html")]
	ResetPassword { reset_url: String },
}

impl Notification {
	/// Fire off this notification for the given profile
	pub async fn fire_for(
		&self,
		profile: &Profile,
		mailer: &Mailer,
	) -> Result<(), Error> {
		todo!()
	}
}
