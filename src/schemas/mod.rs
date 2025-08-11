use serde::de::Visitor;

use crate::Config;

pub mod auth;
pub mod authority;
pub mod image;
pub mod location;
pub mod opening_time;
pub mod pagination;
pub mod profile;
pub mod reservation;
pub mod review;
pub mod tag;
pub mod translation;

pub trait BuildResponse<R> {
	fn build_response(self, config: &Config) -> R;
}

/// A visitor for bounded u32 values.
struct BoundedU32Visitor {
	start: u32,
	end:   u32,
}

impl Visitor<'_> for BoundedU32Visitor {
	type Value = u32;

	/// The expected format for the value.
	fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "a number between {} and {}", self.start, self.end)
	}

	/// Check if the value is within the specified bounds.
	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if (self.start..=self.end).contains(&v) {
			Ok(v)
		} else {
			Err(E::custom(format_args!(
				"invalid value: {v}, expected a number between {} and {}",
				self.start, self.end,
			)))
		}
	}
}

/// Serialize an `Option<Option<T>>` value.
/// Used for dynamic relationship includes in the API.
pub fn ser_includes<S, T>(
	value: &Option<Option<T>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
	T: serde::Serialize,
{
	match value {
		None => serializer.serialize_none(),
		Some(None) => serializer.serialize_some(&None::<T>),
		Some(Some(v)) => v.serialize(serializer),
	}
}
