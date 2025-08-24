use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Copy, DbEnum, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[ExistingTypePath = "crate::sql_types::ProfileState"]
pub enum ProfileState {
	#[default]
	PendingEmailVerification,
	Active,
	Disabled,
}

#[derive(
	Clone, Copy, DbEnum, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[ExistingTypePath = "crate::sql_types::InstitutionCategory"]
pub enum InstitutionCategory {
	#[default]
	Education,
	Organisation,
	Government,
}

impl InstitutionCategory {
	#[must_use]
	pub fn get_variants() -> [&'static str; 3] {
		["education", "organisation", "government"]
	}
}

#[derive(
	Clone, Copy, DbEnum, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[ExistingTypePath = "crate::sql_types::ReservationState"]
pub enum ReservationState {
	#[default]
	Created,
	Cancelled,
	Absent,
	Present,
}
