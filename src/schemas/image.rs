use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponse {
	id:        i32,
	file_path: String,
}
