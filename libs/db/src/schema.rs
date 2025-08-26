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
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
		institution_id -> Nullable<Int4>,
	}
}

diesel::table! {
	authority_profile (id) {
		authority_id -> Int4,
		profile_id -> Int4,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
		id -> Int4,
	}
}

diesel::table! {
	authority_profile_role (authority_profile_id, authority_role_id) {
		authority_profile_id -> Int4,
		authority_role_id -> Int4,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
	}
}

diesel::table! {
	authority_role (id) {
		id -> Int4,
		authority_id -> Int4,
		name -> Text,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	authority_role_permission (authority_role_id, permission_id) {
		authority_role_id -> Int4,
		permission_id -> Int4,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
	}
}

diesel::table! {
	image (id) {
		id -> Int4,
		file_path -> Nullable<Text>,
		uploaded_at -> Timestamp,
		uploaded_by -> Nullable<Int4>,
		image_url -> Nullable<Text>,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::InstitutionCategory;

	institution (id) {
		id -> Int4,
		name_translation_id -> Int4,
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
		created_by -> Int4,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
		category -> InstitutionCategory,
		slug -> Text,
	}
}

diesel::table! {
	institution_profile (id) {
		institution_id -> Int4,
		profile_id -> Int4,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
		id -> Int4,
	}
}

diesel::table! {
	institution_profile_role (institution_profile_id, institution_role_id) {
		institution_profile_id -> Int4,
		institution_role_id -> Int4,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
	}
}

diesel::table! {
	institution_role (id) {
		id -> Int4,
		institution_id -> Int4,
		name -> Text,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	institution_role_permission (institution_role_id, permission_id) {
		institution_role_id -> Int4,
		permission_id -> Int4,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
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
		approved_at -> Nullable<Timestamp>,
		approved_by -> Nullable<Int4>,
		index -> Int4,
	}
}

diesel::table! {
	location_profile (id) {
		location_id -> Int4,
		profile_id -> Int4,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
		id -> Int4,
	}
}

diesel::table! {
	location_profile_role (location_profile_id, location_role_id) {
		location_profile_id -> Int4,
		location_role_id -> Int4,
		added_at -> Timestamp,
		added_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location_role (id) {
		id -> Int4,
		location_id -> Int4,
		name -> Text,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
		updated_at -> Timestamp,
		updated_by -> Nullable<Int4>,
	}
}

diesel::table! {
	location_role_permission (location_role_id, permission_id) {
		location_role_id -> Int4,
		permission_id -> Int4,
		created_at -> Timestamp,
		created_by -> Nullable<Int4>,
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
	permission (id) {
		id -> Int4,
		name -> Text,
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
		created_at -> Timestamp,
		updated_at -> Timestamp,
		confirmed_at -> Nullable<Timestamp>,
		confirmed_by -> Nullable<Int4>,
		state -> ReservationState,
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
diesel::joinable!(authority_profile -> authority (authority_id));
diesel::joinable!(authority_profile_role -> authority_profile (authority_profile_id));
diesel::joinable!(authority_profile_role -> authority_role (authority_role_id));
diesel::joinable!(authority_profile_role -> profile (added_by));
diesel::joinable!(authority_role -> authority (authority_id));
diesel::joinable!(authority_role_permission -> authority_role (authority_role_id));
diesel::joinable!(authority_role_permission -> permission (permission_id));
diesel::joinable!(authority_role_permission -> profile (created_by));
diesel::joinable!(institution -> translation (name_translation_id));
diesel::joinable!(institution_profile -> institution (institution_id));
diesel::joinable!(institution_profile_role -> institution_profile (institution_profile_id));
diesel::joinable!(institution_profile_role -> institution_role (institution_role_id));
diesel::joinable!(institution_profile_role -> profile (added_by));
diesel::joinable!(institution_role -> institution (institution_id));
diesel::joinable!(institution_role_permission -> institution_role (institution_role_id));
diesel::joinable!(institution_role_permission -> permission (permission_id));
diesel::joinable!(institution_role_permission -> profile (created_by));
diesel::joinable!(location -> authority (authority_id));
diesel::joinable!(location_image -> image (image_id));
diesel::joinable!(location_image -> location (location_id));
diesel::joinable!(location_image -> profile (approved_by));
diesel::joinable!(location_profile -> location (location_id));
diesel::joinable!(location_profile_role -> location_profile (location_profile_id));
diesel::joinable!(location_profile_role -> location_role (location_role_id));
diesel::joinable!(location_profile_role -> profile (added_by));
diesel::joinable!(location_role -> location (location_id));
diesel::joinable!(location_role_permission -> location_role (location_role_id));
diesel::joinable!(location_role_permission -> permission (permission_id));
diesel::joinable!(location_role_permission -> profile (created_by));
diesel::joinable!(location_tag -> location (location_id));
diesel::joinable!(location_tag -> tag (tag_id));
diesel::joinable!(opening_time -> location (location_id));
diesel::joinable!(reservation -> opening_time (opening_time_id));
diesel::joinable!(review -> location (location_id));
diesel::joinable!(review -> profile (profile_id));
diesel::joinable!(tag -> translation (name_translation_id));

diesel::allow_tables_to_appear_in_same_query!(
	authority,
	authority_profile,
	authority_profile_role,
	authority_role,
	authority_role_permission,
	image,
	institution,
	institution_profile,
	institution_profile_role,
	institution_role,
	institution_role_permission,
	location,
	location_image,
	location_profile,
	location_profile_role,
	location_role,
	location_role_permission,
	location_tag,
	opening_time,
	permission,
	profile,
	reservation,
	review,
	tag,
	translation,
);
