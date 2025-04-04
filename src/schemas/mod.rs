use serde::{Deserialize, Deserializer};

pub mod location;
pub mod profile;
pub mod translation;

/// Deserialize a value into an `Option<T>`,
/// returning `None` if the value is `null`.
///
/// # Errors
/// If the value cannot be deserialized into `T`, an error is returned.
pub fn ds_patch<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>,
{
	Deserialize::deserialize(deserializer).map(Some)
}
