//! Database model definitions

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate tracing;

use diesel::BoxableExpression;
use diesel::pg::Pg;
use diesel::sql_types::{Bool, Nullable};

mod authority;
mod image;
mod location;
mod opening_time;
mod profile;
mod reservation;
mod tag;
mod translation;

pub mod schema;

pub use authority::*;
pub use image::*;
pub use location::*;
pub use opening_time::*;
pub use profile::*;
pub use reservation::*;
pub use tag::*;
pub use translation::*;

pub type BoxedCondition<S, T = Nullable<Bool>> =
	Box<dyn BoxableExpression<S, Pg, SqlType = T>>;

pub trait ToFilter<S> {
	type SqlType;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType>;
}
