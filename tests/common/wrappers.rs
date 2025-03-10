use std::time::Duration;

use lettre::Address;

use super::TestEnv;

impl TestEnv {
	/// Call a closure and check that it creates no emails
	#[allow(dead_code)]
	pub async fn expect_no_mail<F, R, T>(&self, f: F) -> T
	where
		F: FnOnce() -> R,
		R: Future<Output = T>,
	{
		let outbox_size = { self.stub_mailbox.mailbox.lock().len() };

		let result = f().await;

		// Wait for up to 1 second or until a condvar notification is received
		// to make sure no queued emails are missed
		let mut mailbox = self.stub_mailbox.mailbox.lock();
		if mailbox.len() == outbox_size {
			self.stub_mailbox
				.mail_signal
				.wait_for(&mut mailbox, Duration::from_secs(1));
		}

		assert_eq!(outbox_size, mailbox.len(), "expected no emails to be sent");

		result
	}

	/// Call a closure and check that it creates at least 1 email
	#[allow(dead_code)]
	pub async fn expect_mail<F, R, T>(&self, f: F) -> T
	where
		F: FnOnce() -> R,
		R: Future<Output = T>,
	{
		let outbox_size = { self.stub_mailbox.mailbox.lock().len() };

		let result = f().await;

		// Wait for up to 1 second or until a condvar notification is received
		// to make sure no queued emails are missed
		let mut mailbox = self.stub_mailbox.mailbox.lock();
		if mailbox.len() == outbox_size {
			let wait_res = self
				.stub_mailbox
				.mail_signal
				.wait_for(&mut mailbox, Duration::from_secs(1));

			assert!(!wait_res.timed_out(), "timed out waiting for email");
		}

		assert_eq!(
			mailbox.len(),
			outbox_size + 1,
			"expected an email to be sent"
		);

		result
	}

	/// Call a closure and check that it creates at least 1 email addressed to
	/// the given list of receivers
	#[allow(dead_code)]
	pub async fn expect_mail_to<F, R, T>(&self, receivers: Vec<&str>, f: F) -> T
	where
		F: FnOnce() -> R,
		R: Future<Output = T>,
	{
		let outbox_size = { self.stub_mailbox.mailbox.lock().len() };

		let result = f().await;

		// Wait for up to 1 second or until a condvar notification is received
		// to make sure no queued emails are missed
		let mut mailbox = self.stub_mailbox.mailbox.lock();
		if mailbox.len() == outbox_size {
			let wait_res = self
				.stub_mailbox
				.mail_signal
				.wait_for(&mut mailbox, Duration::from_secs(1));

			assert!(!wait_res.timed_out(), "timed out waiting for email");
		}

		assert_eq!(
			mailbox.len(),
			outbox_size + 1,
			"expected an email to be sent"
		);

		let last_mail = mailbox.last().unwrap();
		let receivers = receivers
			.into_iter()
			.map(|e| e.parse().unwrap())
			.collect::<Vec<Address>>();

		assert_eq!(
			last_mail.envelope().to(),
			receivers,
			"unexpected receivers"
		);

		result
	}
}
