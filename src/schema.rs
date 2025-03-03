// @generated automatically by Diesel CLI.

pub mod sql_types {
	#[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "language"))]
	pub struct Language;

	#[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "user_state"))]
	pub struct UserState;
}

diesel::table! {
	alembic_version (version_num) {
		#[max_length = 32]
		version_num -> Varchar,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::UserState;

	profile (id) {
		id -> Int4,
		username -> Text,
		password_hash -> Text,
		password_reset_token -> Nullable<Text>,
		password_reset_token_expiry -> Nullable<Timestamp>,
		email -> Nullable<Text>,
		pending_email -> Nullable<Text>,
		email_confirmation_token -> Nullable<Text>,
		email_confirmation_token_expiry -> Nullable<Timestamp>,
		admin -> Bool,
		state -> UserState,
		created_at -> Timestamp,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::Language;

	translation (id) {
		id -> Int4,
		language -> Language,
		key -> Uuid,
		text -> Text,
		created_at -> Timestamp,
		updated_at -> Timestamp,
	}
}

diesel::table! {
	user (id) {
		id -> Int8,
		username -> Text,
		email -> Text,
		hashed_password -> Text,
	}
}

diesel::allow_tables_to_appear_in_same_query!(
	alembic_version,
	profile,
	translation,
	user,
);
