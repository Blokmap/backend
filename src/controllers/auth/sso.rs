use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::{Error, OAuthError};
use models::Profile;
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::reqwest::blocking::ClientBuilder;
use openidconnect::reqwest::redirect::Policy;
use openidconnect::{
	AuthenticationFlow,
	AuthorizationCode,
	CsrfToken,
	IssuerUrl,
	Nonce,
	RedirectUrl,
	Scope,
};
use serde::Deserialize;
use time::Duration;

use crate::{Config, DbPool, RedisConn, Session, SsoConfig};

#[must_use]
pub fn make_cookie(
	name: String,
	value: String,
	domain: String,
	lifespan: Duration,
) -> Cookie<'static> {
	let mut cookie = Cookie::new(name, value);

	cookie.set_domain(domain);
	cookie.set_max_age(lifespan);
	cookie.set_http_only(true);
	cookie.set_secure(true);
	cookie.set_same_site(SameSite::Lax);
	cookie.set_path("/");

	cookie
}

#[allow(clippy::missing_panics_doc)]
#[allow(clippy::missing_errors_doc)]
#[instrument(skip(sso_config, jar))]
pub async fn sso_login(
	State(sso_config): State<SsoConfig>,
	Path(provider): Path<String>,
	mut jar: PrivateCookieJar,
) -> Result<impl IntoResponse, Error> {
	warn!("UNSTABLE SSO LOGIN USED");

	if provider != "google" {
		unimplemented!();
	}

	let (auth_url, csrf_state, nonce) =
		tokio::task::block_in_place(move || {
			let issuer_url =
				IssuerUrl::new("https://accounts.google.com".to_string())
					.unwrap();

			let http_client =
				ClientBuilder::new().redirect(Policy::none()).build().unwrap();

			let provider_metadata =
				CoreProviderMetadata::discover(&issuer_url, &http_client)
					.unwrap();

			let client = CoreClient::from_provider_metadata(
				provider_metadata,
				sso_config.google_client_id,
				Some(sso_config.google_client_secret),
			)
			.set_redirect_uri(
				RedirectUrl::new(
					"http://localhost:8000/auth/sso/callback".to_string(),
				)
				.unwrap(),
			);

			client
				.authorize_url(
					AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
					CsrfToken::new_random,
					Nonce::new_random,
				)
				.add_scope(Scope::new("profile".to_string()))
				.add_scope(Scope::new("email".to_string()))
				.url()
		});

	let csrf_cookie = make_cookie(
		"csrf-token".into(),
		csrf_state.into_secret(),
		".localhost".into(),
		Duration::seconds(120),
	);

	let nonce_cookie = make_cookie(
		"nonce-cookie".into(),
		nonce.secret().to_owned(),
		".localhost".into(),
		Duration::seconds(120),
	);

	jar = jar.add(csrf_cookie);
	jar = jar.add(nonce_cookie);

	Ok((jar, Redirect::to(auth_url.as_ref())))
}

#[derive(Clone, Debug, Deserialize)]
pub struct OAuthResponse {
	pub code:  String,
	pub state: String,
}

#[allow(clippy::missing_panics_doc)]
#[allow(clippy::missing_errors_doc)]
#[instrument(skip(config, sso_config, pool, r_conn, jar))]
pub async fn sso_callback(
	State(config): State<Config>,
	State(sso_config): State<SsoConfig>,
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	Query(query): Query<OAuthResponse>,
	mut jar: PrivateCookieJar,
) -> Result<impl IntoResponse, Error> {
	warn!("UNSTABLE SSO LOGIN USED");

	let csrf_cookie =
		jar.get("csrf-token").ok_or(OAuthError::MissingCSRFTokenCookie)?;
	let nonce_cookie =
		jar.get("nonce-cookie").ok_or(OAuthError::MissingNonceCookie)?;

	let csrf_token = csrf_cookie.value().to_owned();
	let nonce = nonce_cookie.value().to_owned();

	jar = jar.remove(csrf_cookie);
	jar = jar.remove(nonce_cookie);

	if csrf_token != query.state {
		return Err(OAuthError::InvalidCSRFToken.into());
	}

	let id_token_claims = tokio::task::block_in_place(move || {
		let issuer_url =
			IssuerUrl::new("https://accounts.google.com".to_string()).unwrap();

		let http_client =
			ClientBuilder::new().redirect(Policy::none()).build().unwrap();

		let provider_metadata =
			CoreProviderMetadata::discover(&issuer_url, &http_client).unwrap();

		let client = CoreClient::from_provider_metadata(
			provider_metadata,
			sso_config.google_client_id,
			Some(sso_config.google_client_secret),
		)
		.set_redirect_uri(
			RedirectUrl::new(
				"http://localhost:8000/auth/sso/callback".to_string(),
			)
			.unwrap(),
		);

		let token_response = client
			.exchange_code(AuthorizationCode::new(query.code))
			.unwrap()
			.request(&http_client)
			.unwrap();

		let id_token_verifier = client.id_token_verifier();

		token_response
			.extra_fields()
			.id_token()
			.unwrap()
			.claims(&id_token_verifier, &Nonce::new(nonce))
			.unwrap()
			.to_owned()
	});

	let conn = pool.get().await?;

	let Some(email) = id_token_claims.email() else {
		return Err(OAuthError::MissingEmailField.into());
	};

	let email = email.to_string();
	let username = id_token_claims.preferred_username().map(|n| n.to_string());
	let profile = Profile::from_sso(email, username, &conn).await?;

	let session =
		Session::create(config.access_token_lifetime, &profile, &mut r_conn)
			.await?;

	let access_token_cookie = session.to_access_token_cookie(
		config.access_token_name,
		config.access_token_lifetime,
		config.production,
	);

	let jar = jar.add(access_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("logged in profile {} from external SSO", profile.id);

	Ok((jar, Json(profile)))
}
