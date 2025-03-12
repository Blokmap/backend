//! Database model definitions

mod location;
mod profile;
mod translation;

pub use location::*;
pub mod ephemeral;
pub use profile::*;
use serde::{Deserialize, Deserializer};
pub use translation::*;

pub fn ds_patch<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>,
{
	Deserialize::deserialize(deserializer).map(Some)
}
