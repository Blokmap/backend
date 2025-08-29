// @generated automatically by Diesel CLI.

pub mod sql_types {
	#[derive(diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "institution_category"))]
	pub struct InstitutionCategory;

	#[derive(diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "profile_state"))]
	pub struct ProfileState;

	#[derive(diesel::sql_types::SqlType)]
	#[diesel(postgres_type(name = "reservation_state"))]
	pub struct ReservationState;
}

diesel::table! {
	authority (id) {
		id -> Int4,
		name -> Text,
		description -> Nullable<Text>,
		institution_id -> Nullable<Int4>,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	authority_member (authority_id, profile_id) {
		authority_id -> Int4,
		profile_id -> Int4,
		authority_role_id -> Nullable<Int4>,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	authority_role (id) {
		id -> Int4,
		authority_id -> Int4,
		name -> Text,
		permissions -> Int8,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	image (id) {
		id -> Int4,
		file_path -> Nullable<Text>,
		image_url -> Nullable<Text>,
		uploaded_at -> Timestamp,
		uploaded_by -> Nullable<Int4>,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::InstitutionCategory;

	institution (id) {
		id -> Int4,
		name_translation_id -> Int4,
		slug -> Text,
		category -> InstitutionCategory,
		email -> Nullable<Text>,
		phone_number -> Nullable<Text>,
		street -> Nullable<Text>,
		number -> Nullable<Text>,
		zip -> Nullable<Text>,
		city -> Nullable<Text>,
		province -> Nullable<Text>,
		#[max_length = 2]
		country -> Nullable<Varchar>,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	institution_member (institution_id, profile_id) {
		institution_id -> Int4,
		profile_id -> Int4,
		institution_role_id -> Nullable<Int4>,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	institution_role (id) {
		id -> Int4,
		institution_id -> Int4,
		name -> Text,
		permissions -> Int8,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location (id) {
		id -> Int4,
		name -> Text,
		authority_id -> Nullable<Int4>,
		description_id -> Int4,
		excerpt_id -> Int4,
		seat_count -> Int4,
		is_reservable -> Bool,
		max_reservation_length -> Nullable<Int4>,
		is_visible -> Bool,
		street -> Text,
		number -> Text,
		zip -> Text,
		city -> Text,
		province -> Text,
		#[max_length = 2]
		country -> Varchar,
		latitude -> Float8,
		longitude -> Float8,
		approved_at -> Nullable<Timestamp>,
		approved_by -> Nullable<Int4>,
		rejected_at -> Nullable<Timestamp>,
		rejected_by -> Nullable<Int4>,
		rejected_reason -> Nullable<Text>,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location_image (location_id, image_id) {
		location_id -> Int4,
		image_id -> Int4,
		index -> Int4,
		approved_at -> Nullable<Timestamp>,
		approved_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location_member (location_id, profile_id) {
		location_id -> Int4,
		profile_id -> Int4,
		location_role_id -> Nullable<Int4>,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location_role (id) {
		id -> Int4,
		location_id -> Int4,
		name -> Text,
		permissions -> Int8,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location_tag (location_id, tag_id) {
		location_id -> Int4,
		tag_id -> Int4,
	}
}

diesel::table! {
	opening_time (id) {
		id -> Int4,
		location_id -> Int4,
		day -> Date,
		start_time -> Time,
		end_time -> Time,
		seat_count -> Nullable<Int4>,
		reservable_from -> Nullable<Timestamp>,
		reservable_until -> Nullable<Timestamp>,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::ProfileState;

	profile (id) {
		id -> Int4,
		username -> Text,
		first_name -> Nullable<Text>,
		last_name -> Nullable<Text>,
		avatar_image_id -> Nullable<Int4>,
		institution_id -> Nullable<Int4>,
		password_hash -> Text,
		password_reset_token -> Nullable<Text>,
		password_reset_token_expiry -> Nullable<Timestamp>,
		email -> Nullable<Text>,
		pending_email -> Nullable<Text>,
		email_confirmation_token -> Nullable<Text>,
		email_confirmation_token_expiry -> Nullable<Timestamp>,
		is_admin -> Bool,
		block_reason -> Nullable<Text>,
		state -> ProfileState,
		created_at -> Timestamp,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
		last_login_at -> Timestamp,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::ReservationState;

	reservation (id) {
		id -> Int4,
		profile_id -> Int4,
		opening_time_id -> Int4,
		base_block_index -> Int4,
		block_count -> Int4,
		state -> ReservationState,
		created_at -> Timestamp,
		updated_at -> Timestamp,
		confirmed_at -> Nullable<Timestamp>,
		confirmed_by -> Nullable<Int4>,
	}
}

diesel::table! {
	review (id) {
		id -> Int4,
		profile_id -> Int4,
		location_id -> Int4,
		rating -> Int4,
		body -> Nullable<Text>,
		created_at -> Timestamp,
		updated_at -> Timestamp,
		hidden_at -> Nullable<Timestamp>,
		hidden_by -> Nullable<Int4>,
	}
}

diesel::table! {
	tag (id) {
		id -> Int4,
		name_translation_id -> Int4,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
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
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::joinable!(authority -> institution (institution_id));
diesel::joinable!(authority_member -> authority (authority_id));
diesel::joinable!(authority_member -> authority_role (authority_role_id));
diesel::joinable!(authority_role -> authority (authority_id));
diesel::joinable!(institution -> translation (name_translation_id));
diesel::joinable!(institution_member -> institution (institution_id));
diesel::joinable!(institution_member -> institution_role (institution_role_id));
diesel::joinable!(institution_role -> institution (institution_id));
diesel::joinable!(location -> authority (authority_id));
diesel::joinable!(location_image -> image (image_id));
diesel::joinable!(location_image -> location (location_id));
diesel::joinable!(location_image -> profile (approved_by));
diesel::joinable!(location_member -> location (location_id));
diesel::joinable!(location_member -> location_role (location_role_id));
diesel::joinable!(location_role -> location (location_id));
diesel::joinable!(location_tag -> location (location_id));
diesel::joinable!(location_tag -> tag (tag_id));
diesel::joinable!(opening_time -> location (location_id));
diesel::joinable!(reservation -> opening_time (opening_time_id));
diesel::joinable!(review -> location (location_id));
diesel::joinable!(tag -> translation (name_translation_id));

diesel::allow_tables_to_appear_in_same_query!(
	authority,
	authority_member,
	authority_role,
	image,
	institution,
	institution_member,
	institution_role,
	location,
	location_image,
	location_member,
	location_role,
	location_tag,
	opening_time,
	profile,
	reservation,
	review,
	tag,
	translation,
);
