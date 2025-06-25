use askama::Template;
use common::Error;
use models::Profile;

use crate::mailer::Mailer;

#[derive(Clone, Debug, Template)]
#[template(path = "confirm_email.html")]
struct ConfirmEmailTemplate {
	confirmation_url: String,
}

#[derive(Clone, Debug, Template)]
#[template(path = "reset_password.html")]
struct ResetPasswordTemplate {
	reset_url: String,
}

impl Mailer {
	/// Send out an email confirmation email
	#[instrument(skip(self))]
	pub(crate) async fn send_confirm_email(
		&self,
		profile: &Profile,
		confirmation_token: &str,
		frontend_url: &str,
	) -> Result<(), Error> {
		let confirmation_url =
			format!("{frontend_url}/confirm_email/{confirmation_token}");

		let body = ConfirmEmailTemplate { confirmation_url };

		let mail = self.try_build_message(
			profile,
			"Confirm your email",
			&body.render()?,
		)?;

		self.send(mail).await?;

		info!("sent new email confirmation email for profile {}", profile.id);

		Ok(())
	}

	/// Send out a password reset email
	#[instrument(skip(self))]
	pub(crate) async fn send_reset_password(
		&self,
		profile: &Profile,
		reset_token: &str,
		frontend_url: &str,
	) -> Result<(), Error> {
		let reset_url = format!("{frontend_url}/reset_password/{reset_token}",);

		let body = ResetPasswordTemplate { reset_url };

		let mail = self.try_build_message(
			profile,
			"Reset your password",
			&body.render()?,
		)?;

		self.send(mail).await?;

		info!("sent password reset email for profile {}", profile.id,);

		Ok(())
	}
}
