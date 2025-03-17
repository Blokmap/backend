use serde::{Deserialize, Deserializer};

pub mod location;
pub mod translation;

pub fn ds_patch<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>,
{
	Deserialize::deserialize(deserializer).map(Some)
}
