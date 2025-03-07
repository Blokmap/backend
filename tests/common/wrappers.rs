use std::sync::Arc;
use std::time::Duration;

use blokmap::mailer::StubMailbox;

#[allow(dead_code)]
pub async fn expect_no_mail<F, R, T>(stub_mailer: Arc<StubMailbox>, f: F) -> T
where
	F: FnOnce() -> R,
	R: Future<Output = T>,
{
	let outbox_size = { stub_mailer.mailbox.lock().len() };

	let result = f().await;

	// Wait for up to 1 second or until a condvar notification is received to
	// make sure no queued emails are missed
	let mut mailbox = stub_mailer.mailbox.lock();
	if mailbox.len() == outbox_size {
		stub_mailer.signal.wait_for(&mut mailbox, Duration::from_secs(1));
	}

	assert_eq!(outbox_size, mailbox.len(), "expected no emails to be sent");

	result
}

#[allow(dead_code)]
pub async fn expect_mail<F, R, T>(stub_mailer: Arc<StubMailbox>, f: F) -> T
where
	F: FnOnce() -> R,
	R: Future<Output = T>,
{
	let outbox_size = { stub_mailer.mailbox.lock().len() };

	let result = f().await;

	tokio::time::sleep(Duration::from_secs(1)).await;

	// Wait for up to 1 second or until a condvar notification is received to
	// make sure no queued emails are missed
	let mut mailbox = stub_mailer.mailbox.lock();
	if mailbox.len() == outbox_size {
		let wait_res =
			stub_mailer.signal.wait_for(&mut mailbox, Duration::from_secs(1));

		assert!(!wait_res.timed_out(), "timed out waiting for email");
	}

	assert_eq!(mailbox.len(), outbox_size + 1, "expected an email to be sent");

	result
}

// #[allow(dead_code)]
// pub async fn expect_mail_to<F, R, T>(
// 	stub_mailer: Arc<StubMailbox>,
// 	receivers: Vec<&str>,
// 	f: F
// ) -> T
// where
// 	F: FnOnce() -> R,
// 	R: Future<Output = T>,
// {
// 	let outbox_size = { stub_mailer.mailbox.lock().len() };

// 	let result = f().await;

// 	// Wait for up to 1 second or until a condvar notification is received to
// 	// make sure no queued emails are missed
// 	let mut mailbox = stub_mailer.mailbox.lock();
// 	if mailbox.len() == outbox_size {
// 		let wait_res =
// 			stub_mailer.signal.wait_for(&mut mailbox, Duration::from_secs(1));

// 		assert!(!wait_res.timed_out(), "timed out waiting for email");
// 	}

// 	assert_eq!(mailbox.len(), outbox_size + 1, "expected an email to be sent");

// 	let last_mail = mailbox.last().unwrap();
// 	let receivers = receivers
// 		.into_iter()
// 		.map(|e| e.parse().unwrap())
// 		.collect::<Vec<Address>>();

// 	assert_eq!(last_mail.envelope().to(), receivers, "unexpected receivers");

// 	result
// }
