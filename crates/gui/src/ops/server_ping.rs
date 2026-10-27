use freya::prelude::Color;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PlayerSample {
	pub name: String,
	pub id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServerPingState {
	Loading,
	Online {
		latency_ms: u64,
		players_online: u32,
		players_max: u32,
		player_sample: Vec<PlayerSample>,
		motd: Option<String>,
		version: Option<String>,
		favicon_bytes: Option<Vec<u8>>,
	},
	Offline {
		reason: Option<String>,
	},
}

#[derive(Deserialize)]
struct StatusServerResponse {
	status: Option<StatusServerStatus>,
}

#[derive(Deserialize)]
struct StatusServerStatus {
	online: Option<bool>,
	latency_ms: Option<u64>,
	motd: Option<String>,
	reason: Option<String>,
	favicon: Option<String>,
	players: Option<StatusServerPlayers>,
	version: Option<StatusServerVersion>,
}

#[derive(Deserialize)]
struct StatusServerPlayers {
	online: Option<u32>,
	max: Option<u32>,
	sample: Option<Vec<PlayerSample>>,
}

#[derive(Deserialize)]
struct StatusServerVersion {
	name: Option<String>,
}

pub async fn ping_server(address: &str) -> ServerPingState {
	if let Some(res) = ping_via_status_server(address).await {
		return res;
	}
	ping_direct_tcp(address).await
}

fn decode_favicon(raw: Option<&str>) -> Option<Vec<u8>> {
	let raw = raw?;
	let data = raw.strip_prefix("data:image/png;base64,").unwrap_or(raw);
	use base64::Engine;
	base64::prelude::BASE64_STANDARD.decode(data).ok()
}

async fn ping_via_status_server(address: &str) -> Option<ServerPingState> {
	let client = reqwest::Client::builder()
		.timeout(Duration::from_millis(3500))
		.build()
		.ok()?;

	let url = format!("http://127.0.0.1:4040/api/ping?host={address}");
	let res = client.get(&url).send().await.ok()?;
	let json = res.json::<StatusServerResponse>().await.ok()?;
	let status = json.status?;

	if status.online.unwrap_or(false) {
		let favicon_bytes = decode_favicon(status.favicon.as_deref());
		let player_sample = status
			.players
			.as_ref()
			.and_then(|p| p.sample.clone())
			.unwrap_or_default();
		Some(ServerPingState::Online {
			latency_ms: status.latency_ms.unwrap_or(1),
			players_online: status.players.as_ref().and_then(|p| p.online).unwrap_or(0),
			players_max: status.players.as_ref().and_then(|p| p.max).unwrap_or(0),
			player_sample,
			motd: status.motd,
			version: status.version.and_then(|v| v.name),
			favicon_bytes,
		})
	} else {
		Some(ServerPingState::Offline {
			reason: status.reason,
		})
	}
}

async fn ping_direct_tcp(address: &str) -> ServerPingState {
	let (host, port) = if let Some((h, p)) = address.split_once(':') {
		(h.to_string(), p.parse::<u16>().unwrap_or(25565))
	} else {
		(address.to_string(), 25565)
	};

	let (target_host, target_port) = resolve_srv(&host, port).await;

	let connect_start = Instant::now();
	let connect_fut = TcpStream::connect((target_host.as_str(), target_port));
	let stream = match tokio::time::timeout(Duration::from_millis(3000), connect_fut).await {
		Ok(Ok(s)) => s,
		_ => {
			return ServerPingState::Offline {
				reason: Some("Connection timed out".to_string()),
			};
		}
	};
	let connect_rtt = connect_start.elapsed().as_millis() as u64;

	let res = perform_slp_handshake(stream, &host, target_port, connect_rtt).await;
	match res {
		Ok(state) => state,
		Err(e) => ServerPingState::Offline { reason: Some(e) },
	}
}

async fn resolve_srv(host: &str, port: u16) -> (String, u16) {
	if port != 25565 || host.parse::<std::net::IpAddr>().is_ok() || host == "localhost" {
		return (host.to_string(), port);
	}

	match tokio::time::timeout(Duration::from_millis(1500), query_dns_srv(host)).await {
		Ok(Some((srv_target, srv_port))) => (srv_target, srv_port),
		_ => (host.to_string(), port),
	}
}

async fn query_dns_srv(host: &str) -> Option<(String, u16)> {
	let query_domain = format!("_minecraft._tcp.{host}");
	let mut qname = Vec::new();
	for label in query_domain.split('.') {
		if label.is_empty() || label.len() > 63 {
			return None;
		}
		qname.push(label.len() as u8);
		qname.extend_from_slice(label.as_bytes());
	}
	qname.push(0);

	let mut packet = Vec::with_capacity(12 + qname.len() + 4);
	packet.extend_from_slice(&[
		0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	]);
	packet.extend_from_slice(&qname);
	packet.extend_from_slice(&[0x00, 0x21, 0x00, 0x01]);

	let socket = UdpSocket::bind("0.0.0.0:0").await.ok()?;
	let nameservers = get_nameservers();

	let mut buf = [0u8; 2048];
	for ns in nameservers {
		if socket.send_to(&packet, &ns).await.is_ok() {
			if let Ok(Ok((len, _))) =
				tokio::time::timeout(Duration::from_millis(600), socket.recv_from(&mut buf)).await
			{
				if let Some(res) = parse_dns_srv_response(&buf[..len]) {
					return Some(res);
				}
			}
		}
	}
	None
}

fn get_nameservers() -> Vec<String> {
	let mut list = Vec::new();
	if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
		for line in content.lines() {
			let line = line.trim();
			if line.starts_with("nameserver") {
				if let Some(ip) = line.split_whitespace().nth(1) {
					list.push(format!("{ip}:53"));
				}
			}
		}
	}
	if list.is_empty() {
		list.push("1.1.1.1:53".to_string());
		list.push("8.8.8.8:53".to_string());
	}
	list
}

fn parse_dns_srv_response(data: &[u8]) -> Option<(String, u16)> {
	if data.len() < 12 {
		return None;
	}
	let ancount = u16::from_be_bytes([data[6], data[7]]);
	if ancount == 0 {
		return None;
	}
	let mut idx = 12;
	while idx < data.len() {
		let len = data[idx] as usize;
		if len == 0 {
			idx += 1;
			break;
		}
		if (len & 0xC0) == 0xC0 {
			idx += 2;
			break;
		}
		idx += 1 + len;
	}
	idx += 4;

	for _ in 0..ancount {
		if idx >= data.len() {
			break;
		}
		idx = skip_dns_name(data, idx)?;
		if idx + 10 > data.len() {
			break;
		}
		let rtype = u16::from_be_bytes([data[idx], data[idx + 1]]);
		let rdlen = u16::from_be_bytes([data[idx + 8], data[idx + 9]]) as usize;
		idx += 10;
		if rtype == 33 && rdlen >= 7 && idx + rdlen <= data.len() {
			let port = u16::from_be_bytes([data[idx + 4], data[idx + 5]]);
			let target = parse_dns_name(data, idx + 6)?;
			if !target.is_empty() {
				return Some((target.trim_end_matches('.').to_string(), port));
			}
		}
		idx += rdlen;
	}
	None
}

fn skip_dns_name(data: &[u8], mut idx: usize) -> Option<usize> {
	while idx < data.len() {
		let b = data[idx];
		if b == 0 {
			return Some(idx + 1);
		}
		if (b & 0xC0) == 0xC0 {
			return Some(idx + 2);
		}
		idx += 1 + (b as usize);
	}
	None
}

fn parse_dns_name(data: &[u8], mut offset: usize) -> Option<String> {
	let mut parts = Vec::new();
	let mut count = 0;
	while offset < data.len() && count < 64 {
		count += 1;
		let b = data[offset];
		if b == 0 {
			break;
		}
		if (b & 0xC0) == 0xC0 {
			if offset + 1 >= data.len() {
				return None;
			}
			let ptr = ((u16::from_be_bytes([b, data[offset + 1]])) & 0x3FFF) as usize;
			offset = ptr;
		} else {
			let len = b as usize;
			offset += 1;
			if offset + len > data.len() {
				return None;
			}
			let part = std::str::from_utf8(&data[offset..offset + len]).ok()?;
			parts.push(part);
			offset += len;
		}
	}
	Some(parts.join("."))
}

async fn perform_slp_handshake(
	mut stream: TcpStream,
	host: &str,
	port: u16,
	connect_rtt: u64,
) -> Result<ServerPingState, String> {
	let mut handshake = Vec::new();
	handshake.push(0x00);
	write_varint(&mut handshake, 47);
	write_varint(&mut handshake, host.len() as i32);
	handshake.extend_from_slice(host.as_bytes());
	handshake.extend_from_slice(&port.to_be_bytes());
	write_varint(&mut handshake, 1);

	let mut packet = Vec::new();
	write_varint(&mut packet, handshake.len() as i32);
	packet.extend_from_slice(&handshake);
	packet.push(0x01);
	packet.push(0x00);

	stream.write_all(&packet).await.map_err(|e| e.to_string())?;

	let _packet_len = read_varint(&mut stream).await?;
	let packet_id = read_varint(&mut stream).await?;
	if packet_id != 0 {
		return Err("Invalid packet id".to_string());
	}
	let str_len = read_varint(&mut stream).await?;
	if str_len <= 0 || str_len > 1048576 {
		return Err("Invalid length".to_string());
	}

	let mut buf = vec![0u8; str_len as usize];
	stream
		.read_exact(&mut buf)
		.await
		.map_err(|e| e.to_string())?;

	let ping_start = Instant::now();
	let ping_packet = [0x09u8, 0x01, 0, 0, 0, 0, 0, 0, 0, 1];
	let latency = match stream.write_all(&ping_packet).await {
		Ok(_) => {
			let pong_fut = async {
				let _len = read_varint(&mut stream).await?;
				let id = read_varint(&mut stream).await?;
				if id != 1 {
					return Err("Invalid pong id".to_string());
				}
				let mut pong_buf = [0u8; 8];
				stream
					.read_exact(&mut pong_buf)
					.await
					.map_err(|e| e.to_string())?;
				Ok::<(), String>(())
			};
			match tokio::time::timeout(Duration::from_millis(1500), pong_fut).await {
				Ok(Ok(())) => ping_start.elapsed().as_millis() as u64,
				_ => connect_rtt,
			}
		}
		Err(_) => connect_rtt,
	};

	if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&buf) {
		let players = json.get("players");
		let online = players
			.and_then(|p| p.get("online"))
			.and_then(|o| o.as_u64())
			.unwrap_or(0) as u32;
		let max = players
			.and_then(|p| p.get("max"))
			.and_then(|m| m.as_u64())
			.unwrap_or(0) as u32;

		let player_sample = players
			.and_then(|p| p.get("sample"))
			.and_then(|s| serde_json::from_value::<Vec<PlayerSample>>(s.clone()).ok())
			.unwrap_or_default();

		let motd = json.get("description").map(extract_motd);

		let version = json
			.get("version")
			.and_then(|v| v.get("name"))
			.and_then(|n| n.as_str())
			.map(|s| s.to_string());

		let favicon_bytes = json
			.get("favicon")
			.and_then(|f| f.as_str())
			.and_then(|s| decode_favicon(Some(s)));

		Ok(ServerPingState::Online {
			latency_ms: latency.max(1),
			players_online: online,
			players_max: max,
			player_sample,
			motd,
			version,
			favicon_bytes,
		})
	} else {
		Ok(ServerPingState::Online {
			latency_ms: latency.max(1),
			players_online: 0,
			players_max: 0,
			player_sample: Vec::new(),
			motd: None,
			version: None,
			favicon_bytes: None,
		})
	}
}

pub fn extract_motd(val: &serde_json::Value) -> String {
	let mut out = String::new();
	append_motd_value(val, &mut out);
	out.trim().to_string()
}

fn append_motd_value(val: &serde_json::Value, out: &mut String) {
	if let Some(s) = val.as_str() {
		out.push_str(s);
	} else if let Some(arr) = val.as_array() {
		for item in arr {
			append_motd_value(item, out);
		}
	} else if let Some(obj) = val.as_object() {
		if let Some(color) = obj.get("color").and_then(|c| c.as_str()) {
			if let Some(code) = minecraft_color_name_to_code(color) {
				out.push('§');
				out.push(code);
			} else if color.starts_with('#') && color.len() == 7 {
				out.push('§');
				out.push_str(color);
			}
		}
		if obj.get("bold").and_then(|b| b.as_bool()).unwrap_or(false) {
			out.push_str("§l");
		}
		if obj.get("italic").and_then(|b| b.as_bool()).unwrap_or(false) {
			out.push_str("§o");
		}
		if obj.get("underlined").and_then(|b| b.as_bool()).unwrap_or(false) {
			out.push_str("§n");
		}
		if obj.get("strikethrough").and_then(|b| b.as_bool()).unwrap_or(false) {
			out.push_str("§m");
		}
		if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
			out.push_str(text);
		}
		if let Some(extra) = obj.get("extra").and_then(|e| e.as_array()) {
			for item in extra {
				append_motd_value(item, out);
			}
		}
	}
}

fn minecraft_color_name_to_code(name: &str) -> Option<char> {
	match name.to_ascii_lowercase().as_str() {
		"black" => Some('0'),
		"dark_blue" => Some('1'),
		"dark_green" => Some('2'),
		"dark_aqua" => Some('3'),
		"dark_red" => Some('4'),
		"dark_purple" => Some('5'),
		"gold" => Some('6'),
		"gray" => Some('7'),
		"dark_gray" => Some('8'),
		"blue" => Some('9'),
		"green" => Some('a'),
		"aqua" => Some('b'),
		"red" => Some('c'),
		"light_purple" => Some('d'),
		"yellow" => Some('e'),
		"white" => Some('f'),
		_ => None,
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormattedSpan {
	pub text: String,
	pub color: Option<Color>,
	pub bold: bool,
	pub italic: bool,
	pub underline: bool,
	pub strikethrough: bool,
}

pub fn parse_minecraft_formatting(input: &str, default_color: Color) -> Vec<FormattedSpan> {
	let mut spans = Vec::new();
	let mut current_text = String::new();
	let mut current_color: Option<Color> = None;
	let mut bold = false;
	let mut italic = false;
	let mut underline = false;
	let mut strikethrough = false;

	let chars: Vec<char> = input.chars().collect();
	let mut i = 0;

	while i < chars.len() {
		let c = chars[i];
		if (c == '§' || c == '&') && i + 1 < chars.len() {
			let next_c = chars[i + 1];

			// 1) §x§r§r§g§g§b§b
			if (next_c == 'x' || next_c == 'X')
				&& i + 13 < chars.len()
				&& chars[i + 2] == '§'
				&& chars[i + 4] == '§'
				&& chars[i + 6] == '§'
				&& chars[i + 8] == '§'
				&& chars[i + 10] == '§'
				&& chars[i + 12] == '§'
			{
				let hex_str: String = [
					chars[i + 3],
					chars[i + 5],
					chars[i + 7],
					chars[i + 9],
					chars[i + 11],
					chars[i + 13],
				]
				.iter()
				.collect();
				if let Ok(rgb) = u32::from_str_radix(&hex_str, 16) {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					let r = ((rgb >> 16) & 0xFF) as u8;
					let g = ((rgb >> 8) & 0xFF) as u8;
					let b = (rgb & 0xFF) as u8;
					current_color = Some(Color::from_rgb(r, g, b));
					bold = false;
					italic = false;
					underline = false;
					strikethrough = false;
					i += 14;
					continue;
				}
			}

			// 2) §#rrggbb or &#rrggbb
			if next_c == '#' && i + 7 < chars.len() {
				let hex_str: String = chars[i + 2..i + 8].iter().collect();
				if let Ok(rgb) = u32::from_str_radix(&hex_str, 16) {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					let r = ((rgb >> 16) & 0xFF) as u8;
					let g = ((rgb >> 8) & 0xFF) as u8;
					let b = (rgb & 0xFF) as u8;
					current_color = Some(Color::from_rgb(r, g, b));
					bold = false;
					italic = false;
					underline = false;
					strikethrough = false;
					i += 8;
					continue;
				}
			}

			macro_rules! color_code {
				($r:expr, $g:expr, $b:expr) => {{
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					current_color = Some(Color::from_rgb($r, $g, $b));
					bold = false;
					italic = false;
					underline = false;
					strikethrough = false;
					i += 2;
					continue;
				}};
			}

			match next_c.to_ascii_lowercase() {
				'0' => color_code!(0, 0, 0),
				'1' => color_code!(0, 0, 170),
				'2' => color_code!(0, 170, 0),
				'3' => color_code!(0, 170, 170),
				'4' => color_code!(170, 0, 0),
				'5' => color_code!(170, 0, 170),
				'6' => color_code!(255, 170, 0),
				'7' => color_code!(170, 170, 170),
				'8' => color_code!(85, 85, 85),
				'9' => color_code!(85, 85, 255),
				'a' => color_code!(85, 255, 85),
				'b' => color_code!(85, 255, 255),
				'c' => color_code!(255, 85, 85),
				'd' => color_code!(255, 85, 255),
				'e' => color_code!(255, 255, 85),
				'f' => color_code!(255, 255, 255),
				'k' => { i += 2; continue; }
				'l' => {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					bold = true;
					i += 2;
					continue;
				}
				'm' => {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					strikethrough = true;
					i += 2;
					continue;
				}
				'n' => {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					underline = true;
					i += 2;
					continue;
				}
				'o' => {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					italic = true;
					i += 2;
					continue;
				}
				'r' => {
					if !current_text.is_empty() {
						spans.push(FormattedSpan {
							text: std::mem::take(&mut current_text),
							color: current_color.or(Some(default_color)),
							bold,
							italic,
							underline,
							strikethrough,
						});
					}
					current_color = None;
					bold = false;
					italic = false;
					underline = false;
					strikethrough = false;
					i += 2;
					continue;
				}
				_ => {}
			}
		}

		if c == '\n' {
			current_text.push(' ');
		} else if !c.is_control() {
			current_text.push(c);
		}
		i += 1;
	}

	if !current_text.is_empty() {
		spans.push(FormattedSpan {
			text: current_text,
			color: current_color.or(Some(default_color)),
			bold,
			italic,
			underline,
			strikethrough,
		});
	}

	spans
}

fn write_varint(buf: &mut Vec<u8>, mut val: i32) {
	loop {
		if (val & !0x7F) == 0 {
			buf.push(val as u8);
			return;
		}
		buf.push(((val & 0x7F) | 0x80) as u8);
		val >>= 7;
	}
}

async fn read_varint(stream: &mut TcpStream) -> Result<i32, String> {
	let mut num_read = 0;
	let mut result = 0;
	loop {
		let mut buf = [0u8; 1];
		stream
			.read_exact(&mut buf)
			.await
			.map_err(|e| e.to_string())?;
		let read = buf[0];
		let value = (read & 0x7F) as i32;
		result |= value << (7 * num_read);
		num_read += 1;
		if num_read > 5 {
			return Err("VarInt too big".to_string());
		}
		if (read & 0x80) == 0 {
			break;
		}
	}
	Ok(result)
}

#[allow(dead_code)]
fn clean_mc_formatting(text: &str) -> String {
	let mut out = String::new();
	let mut chars = text.chars().peekable();
	while let Some(c) = chars.next() {
		if c == '§' {
			let _ = chars.next();
		} else if c.is_control() {
			out.push(' ');
		} else {
			out.push(c);
		}
	}
	out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	#[ignore = "Requires external internet connectivity"]
	async fn test_ping_live_servers() {
		let res_silverpeek = ping_server("mc.silverpeek.ru").await;
		match res_silverpeek {
			ServerPingState::Online {
				latency_ms,
				players_online,
				favicon_bytes,
				..
			} => {
				println!(
					"mc.silverpeek.ru online: {latency_ms}ms, {players_online} players, has_favicon: {}",
					favicon_bytes.is_some()
				);
				assert!(latency_ms > 0 && latency_ms < 100);
			}
			ServerPingState::Offline { reason } => {
				panic!("mc.silverpeek.ru failed: {:?}", reason);
			}
			_ => panic!("Expected mc.silverpeek.ru to be online"),
		}

		let res_bixland = ping_server("play.bixland.ru").await;
		match res_bixland {
			ServerPingState::Online { latency_ms, .. } => {
				println!("play.bixland.ru online: {latency_ms}ms");
				assert!(latency_ms > 0 && latency_ms < 100);
			}
			ServerPingState::Offline { reason } => {
				panic!("play.bixland.ru failed: {:?}", reason);
			}
			_ => panic!("Expected play.bixland.ru to be online"),
		}

		let res_ip = ping_server("161.104.32.97").await;
		match res_ip {
			ServerPingState::Online { latency_ms, .. } => {
				println!("161.104.32.97 online: {latency_ms}ms");
				assert!(latency_ms > 0 && latency_ms < 100);
			}
			_ => panic!("Expected 161.104.32.97 to be online"),
		}
	}
	#[test]
	fn test_minecraft_formatting_parser() {
		let default_col = Color::from_rgb(200, 200, 200);
		let spans = parse_minecraft_formatting("§6Gold §aGreen §lBoldGreen§r Reset", default_col);
		assert_eq!(spans.len(), 4);
		assert_eq!(spans[0].text, "Gold ");
		assert_eq!(spans[0].color, Some(Color::from_rgb(255, 170, 0)));
		assert!(!spans[0].bold);

		assert_eq!(spans[1].text, "Green ");
		assert_eq!(spans[1].color, Some(Color::from_rgb(85, 255, 85)));
		assert!(!spans[1].bold);

		assert_eq!(spans[2].text, "BoldGreen");
		assert_eq!(spans[2].color, Some(Color::from_rgb(85, 255, 85)));
		assert!(spans[2].bold);

		assert_eq!(spans[3].text, " Reset");
		assert_eq!(spans[3].color, Some(default_col));
		assert!(!spans[3].bold);

		// Hex formatting test: §#ff5500Custom
		let hex_spans = parse_minecraft_formatting("§#ff5500Custom", default_col);
		assert_eq!(hex_spans.len(), 1);
		assert_eq!(hex_spans[0].text, "Custom");
		assert_eq!(hex_spans[0].color, Some(Color::from_rgb(255, 85, 0)));
	}
}
