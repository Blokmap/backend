use std::sync::Arc;

use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, SmtpTransport, Transport};
use parking_lot::{Condvar, Mutex};
use tokio::sync::mpsc;

use crate::{Config, Error};

/// A basic interface to send email messages
#[derive(Clone, Debug)]
pub struct Mailer {
	from:       Address,
	send_queue: mpsc::Sender<Message>,
}

/// A fake mailbox to keep track of mails sent in tests
#[derive(Default)]
pub struct StubMailbox {
	pub mailbox: Mutex<Vec<Message>>,
	pub signal:  Condvar,
}

impl Mailer {
	/// Create a new mailer
	///
	/// # Panics
	/// Panics if a stub mailer is expected but not provied
	#[must_use]
	pub fn new(config: &Config, stub_mailer: Option<Arc<StubMailbox>>) -> Self {
		let (tx, rx) = mpsc::channel(config.email_queue_size);

		if config.email_smtp_server == "stub" {
			assert!(stub_mailer.is_some(), "MISSING STUB MAILER");

			tokio::spawn(Self::start_stub_sender(rx, stub_mailer.unwrap()));
		} else {
			tokio::spawn(Self::start_smtp_sender(
				rx,
				config.email_address.clone(),
				config.email_smtp_server.clone(),
				config.email_smtp_password.clone(),
			));
		}

		Self { from: config.email_address.clone(), send_queue: tx }
	}

	/// Try to build an email [`Message`]
	///
	/// # Errors
	/// Fails if the receiver or body cannot be parsed
	pub fn try_build_message(
		&self,
		receiver: impl TryInto<Mailbox, Error = impl Into<Error>>,
		subject: &str,
		body: &str,
	) -> Result<Message, Error> {
		Ok(Message::builder()
			.from(Mailbox::new(None, self.from.clone()))
			.to(receiver.try_into().map_err(Into::into)?)
			.subject(subject)
			.body(body.to_string())?)
	}

	/// Try to send a message
	///
	/// # Errors
	/// Fails if the mail queue is full
	pub fn try_send(&self, message: Message) -> Result<(), Error> {
		info!("trysent");
		Ok(self.send_queue.try_send(message)?)
	}

	/// Send a message and block if the mail queue is full
	///
	/// # Errors
	/// Fails if the other end of the mail queue was unexpectedly closed
	pub async fn send(&self, message: Message) -> Result<(), Error> {
		info!("sent");
		Ok(self.send_queue.send(message).await?)
	}

	/// Start an infinitely looping stub sender thread
	#[instrument(skip_all)]
	async fn start_stub_sender(
		mut rx: mpsc::Receiver<Message>,
		stub_mailer: Arc<StubMailbox>,
	) -> impl Send + 'static {
		while let Some(mail) = rx.recv().await {
			let mail_pretty =
				String::from_utf8_lossy(&mail.formatted()).to_string();

			info!("received");

			{
				let mut mailbox = stub_mailer.mailbox.lock();
				mailbox.push(mail);
				stub_mailer.signal.notify_all();
			}

			info!(
				target: "[STUB_MAILER]",
				"sent email:\n{}\n",
				mail_pretty
			);

			tokio::time::sleep(std::time::Duration::from_millis(500)).await;
		}
	}

	/// Start an infinitely looping smtp sender thread
	#[instrument(skip_all)]
	async fn start_smtp_sender(
		mut rx: mpsc::Receiver<Message>,
		address: Address,
		server: String,
		password: String,
	) -> impl Send + 'static {
		let transport = SmtpTransport::starttls_relay(&server)
			.expect("STARTTLS ERROR")
			.credentials(Credentials::new(address.to_string(), password))
			.build();

		match transport.test_connection() {
			Ok(_) => (),
			Err(e) => panic!("SMTP CONNECTION FAILED: {e:?}"),
		}

		while let Some(mail) = rx.recv().await {
			match transport.send(&mail) {
				Ok(res) => info!("sent email: {res:?}"),
				Err(e) => error!("error sending email: {e:?}"),
			}

			tokio::time::sleep(std::time::Duration::from_secs(1)).await;
		}
	}
}
