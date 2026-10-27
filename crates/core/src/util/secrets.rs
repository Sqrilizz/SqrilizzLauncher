use sqrilizz_auth::mc::ClientId;

pub fn get_ms_client_id() -> ClientId {
	if let Ok(id) = std::env::var("SQRILIZZ_MS_CLIENT_ID") {
		let id = id.trim();
		if !id.is_empty() {
			return ClientId::new(id.to_string());
		}
	}
	if let Ok(home) = std::env::var("HOME") {
		let path = std::path::Path::new(&home).join(".config/sqrilizz/ms_client_id");
		if let Ok(id) = std::fs::read_to_string(path) {
			let id = id.trim();
			if !id.is_empty() {
				return ClientId::new(id.to_string());
			}
		}
	}
	if let Ok(appdata) = std::env::var("APPDATA") {
		let path = std::path::Path::new(&appdata).join("sqrilizz/sqrilizz/config/ms_client_id");
		if let Ok(id) = std::fs::read_to_string(path) {
			let id = id.trim();
			if !id.is_empty() {
				return ClientId::new(id.to_string());
			}
		}
	}
	ClientId::new(get_raw_ms_client_id().to_string())
}

const fn get_raw_ms_client_id() -> &'static str {
	match option_env!("SQRILIZZ_MS_CLIENT_ID") {
		Some(id) => id,
		None => "402abc71-43fb-45c1-b230-e7fc9d4485fe",
	}
}
