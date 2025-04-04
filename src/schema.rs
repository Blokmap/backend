// @generated automatically by Diesel CLI.

pub mod sql_types {
	#[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "profile_state"))]
	pub struct ProfileState;
}

diesel::table! {
	location (id) {
		id -> Int4,
		name -> Text,
		description_id -> Int4,
		excerpt_id -> Int4,
		seat_count -> Int4,
		is_reservable -> Bool,
		is_visible -> Bool,
		street -> Text,
		number -> Text,
		zip -> Text,
		city -> Text,
		province -> Text,
		latitude -> Float8,
		longitude -> Float8,
		created_by_id -> Int4,
		approved_by_id -> Nullable<Int4>,
		approved_at -> Nullable<Timestamp>,
		created_at -> Timestamp,
		updated_at -> Timestamp,
	}
}

diesel::table! {
	opening_time (id) {
		id -> Int4,
		location_id -> Int4,
		start_time -> Timestamptz,
		end_time -> Timestamptz,
		seat_count -> Nullable<Int4>,
		is_reservable -> Nullable<Bool>,
		created_at -> Timestamptz,
		updated_at -> Timestamptz,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::ProfileState;

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
		state -> ProfileState,
		created_at -> Timestamp,
		last_login_at -> Timestamp,
	}
}

diesel::table! {
	translation (id) {
		id -> Int4,
		nl -> Nullable<Text>,
		en -> Nullable<Text>,
		fr -> Nullable<Text>,
		de -> Nullable<Text>,
		created_at -> Timestamp,
		updated_at -> Timestamp,
	}
}

diesel::joinable!(opening_time -> location (location_id));

diesel::allow_tables_to_appear_in_same_query!(
	location,
	opening_time,
	profile,
	translation,
);
