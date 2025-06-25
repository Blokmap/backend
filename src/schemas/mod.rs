use serde::de::Visitor;

pub mod auth;
pub mod authority;
pub mod image;
pub mod location;
pub mod notification;
pub mod opening_time;
pub mod pagination;
pub mod profile;
pub mod reservation;
pub mod review;
pub mod tag;
pub mod translation;

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
