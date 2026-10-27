use std::{
	collections::HashMap,
	fs,
	io::Cursor,
	sync::{Arc, LazyLock, Mutex},
	time::Duration,
};

use freya::prelude::*;
use image::{AnimationDecoder, ImageFormat, codecs::gif::GifDecoder, imageops::FilterType};

use crate::util::assets::get_instance_icon;

#[derive(Clone)]
pub struct GifFrame {
	pub delay_ms: u64,
	pub bytes: Bytes,
}

#[derive(Clone)]
enum CacheEntry {
	Loading,
	Ready(Arc<Vec<GifFrame>>),
	Failed,
}

static GIF_CACHE: LazyLock<Mutex<HashMap<String, CacheEntry>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn is_gif_path(path_str: &str) -> bool {
	let lower = path_str.to_lowercase();
	if lower.ends_with(".gif") {
		return true;
	}
	let clean = path_str.strip_prefix("file://").unwrap_or(path_str);
	if let Ok(mut file) = fs::File::open(clean) {
		use std::io::Read;
		let mut magic = [0u8; 6];
		if file.read_exact(&mut magic).is_ok() {
			return &magic[..3] == b"GIF";
		}
	}
	false
}

pub fn get_cached_gif(path_str: &str) -> Option<Arc<Vec<GifFrame>>> {
	if let Ok(cache) = GIF_CACHE.lock() {
		if let Some(CacheEntry::Ready(frames)) = cache.get(path_str) {
			return Some(frames.clone());
		}
	}
	None
}

pub async fn get_or_load_gif(path_str: &str) -> Option<Arc<Vec<GifFrame>>> {
	{
		if let Ok(mut cache) = GIF_CACHE.lock() {
			match cache.get(path_str) {
				Some(CacheEntry::Ready(frames)) => return Some(frames.clone()),
				Some(CacheEntry::Loading) => return None,
				Some(CacheEntry::Failed) => return None,
				None => {
					cache.insert(path_str.to_string(), CacheEntry::Loading);
				}
			}
		}
	}

	let path_owned = path_str.to_string();
	let decoded = tokio::task::spawn_blocking(move || decode_gif_internal(&path_owned))
		.await
		.ok()
		.flatten();

	if let Ok(mut cache) = GIF_CACHE.lock() {
		if let Some(ref frames) = decoded {
			cache.insert(path_str.to_string(), CacheEntry::Ready(frames.clone()));
		} else {
			cache.insert(path_str.to_string(), CacheEntry::Failed);
		}
	}

	decoded
}

fn decode_gif_internal(path_str: &str) -> Option<Arc<Vec<GifFrame>>> {
	let clean = path_str.strip_prefix("file://").unwrap_or(path_str);
	let data = fs::read(clean).ok()?;
	let decoder = GifDecoder::new(Cursor::new(&data)).ok()?;
	let raw_frames = decoder.into_frames().collect_frames().ok()?;
	if raw_frames.is_empty() {
		return None;
	}

	let total = raw_frames.len();
	let step = if total > 80 { 2 } else { 1 };

	let mut frames = Vec::with_capacity(total / step + 1);
	for (idx, frame) in raw_frames.into_iter().enumerate() {
		if idx % step != 0 {
			continue;
		}
		let (num, denom) = frame.delay().numer_denom_ms();
		let raw_delay = if denom == 0 {
			100
		} else {
			(num / denom) as u64
		};
		let delay_ms = (raw_delay * step as u64).max(25);

		let buf = frame.into_buffer();
		let resized = if buf.width() > 128 || buf.height() > 128 {
			image::imageops::resize(&buf, 128, 128, FilterType::Nearest)
		} else {
			buf
		};

		let mut png_bytes = Vec::new();
		if resized
			.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
			.is_ok()
		{
			frames.push(GifFrame {
				delay_ms,
				bytes: Bytes::from(png_bytes),
			});
		}
	}

	if frames.is_empty() {
		return None;
	}

	Some(Arc::new(frames))
}

#[derive(Clone, PartialEq)]
pub struct AnimatedIconViewer {
	pub icon: Option<String>,
	pub width: Size,
	pub height: Size,
	pub corner_radius: CornerRadius,
	pub image_cover: Option<ImageCover>,
	pub aspect_ratio: Option<AspectRatio>,
}

impl AnimatedIconViewer {
	pub fn new(icon: Option<String>) -> Self {
		Self {
			icon,
			width: Size::fill(),
			height: Size::fill(),
			corner_radius: CornerRadius::default(),
			image_cover: None,
			aspect_ratio: None,
		}
	}

	pub fn width(mut self, width: Size) -> Self {
		self.width = width;
		self
	}

	pub fn height(mut self, height: Size) -> Self {
		self.height = height;
		self
	}

	pub fn expanded(mut self) -> Self {
		self.width = Size::fill();
		self.height = Size::fill();
		self
	}

	pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
		self.corner_radius = corner_radius.into();
		self
	}

	pub fn image_cover(mut self, image_cover: ImageCover) -> Self {
		self.image_cover = Some(image_cover);
		self
	}

	pub fn aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
		self.aspect_ratio = Some(aspect_ratio);
		self
	}
}

impl Component for AnimatedIconViewer {
	fn render(&self) -> impl IntoElement {
		let frame_idx = use_state(|| 0usize);
		let is_gif = self.icon.as_deref().map(is_gif_path).unwrap_or(false);

		let icon_clone = self.icon.clone();
		let mut frame_idx_anim = frame_idx;
		use_future(move || {
			let icon_str = icon_clone.clone();
			async move {
				if !is_gif {
					return;
				}
				let path = match icon_str {
					Some(p) => p,
					None => return,
				};
				loop {
					let maybe_frames = get_or_load_gif(&path).await;
					if let Some(frames) = maybe_frames {
						if frames.len() <= 1 {
							return;
						}
						let mut current = 0usize;
						loop {
							let delay = frames[current].delay_ms;
							tokio::time::sleep(Duration::from_millis(delay)).await;
							current = (current + 1) % frames.len();
							frame_idx_anim.set(current);
						}
					}
					tokio::time::sleep(Duration::from_millis(200)).await;
				}
			}
		});

		let cached_frames = self.icon.as_deref().and_then(get_cached_gif);

		let mut viewer = if let Some(ref f) = cached_frames {
			let idx = (*frame_idx.read()).min(f.len().saturating_sub(1));
			let frame = &f[idx];
			let path_key = self.icon.as_deref().unwrap_or_default();
			let source: ImageSource = (format!("{path_key}:{idx}"), frame.bytes.clone()).into();
			ImageViewer::new(source)
		} else {
			ImageViewer::new(get_instance_icon(self.icon.as_deref()))
		};

		viewer = viewer
			.width(self.width.clone())
			.height(self.height.clone())
			.corner_radius(self.corner_radius);

		if let Some(cover) = self.image_cover.clone() {
			viewer = viewer.image_cover(cover);
		}

		if let Some(ratio) = self.aspect_ratio.clone() {
			viewer = viewer.aspect_ratio(ratio);
		}

		viewer
	}
}
