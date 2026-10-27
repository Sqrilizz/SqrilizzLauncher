use std::path::PathBuf;

use freya::{
	components::{ImageSource, ImageViewer, Url},
	elements::extensions::ContainerSizeExt,
	prelude::Size,
};
use sqrilizz::shared::{loaders::Loader, pkg::PackageKind};

pub static DEFAULT_INSTANCE: &[u8] = include_bytes!("../assets/images/default_instance.png");
pub static DEFAULT_SKIN: &[u8] = include_bytes!("../assets/images/default_skin.png");
pub static SPLASH: &[u8] = include_bytes!("../assets/images/splash.png");
#[allow(dead_code)]
pub static SPLASH2: &[u8] = include_bytes!("../assets/images/splash2.png");
#[allow(dead_code)]
pub static SPLASH3: &[u8] = include_bytes!("../assets/images/splash3.png");
#[allow(dead_code)]
pub static SPLASH4: &[u8] = include_bytes!("../assets/images/splash4.png");
#[allow(dead_code)]
pub static SPLASH5: &[u8] = include_bytes!("../assets/images/splash5.png");
#[allow(dead_code)]
pub static LOGO_LARGE: &[u8] = include_bytes!("../assets/images/logo_large.png");
pub static FABRIC: &[u8] = include_bytes!("../assets/images/fabric.png");
pub static FOLIA: &[u8] = include_bytes!("../assets/images/folia.png");
pub static FORGE: &[u8] = include_bytes!("../assets/images/forge.png");
pub static MINECRAFT: &[u8] = include_bytes!("../assets/images/minecraft.png");
pub static NEOFORGED: &[u8] = include_bytes!("../assets/images/neoforge.png");
pub static PAPER: &[u8] = include_bytes!("../assets/images/paper.png");
pub static QUILT: &[u8] = include_bytes!("../assets/images/quilt.png");
pub static SPONGE: &[u8] = include_bytes!("../assets/images/sponge.png");

pub fn get_instance_icon(icon: Option<&str>) -> ImageSource {
	let default = ("default-instance", DEFAULT_INSTANCE);

	if let Some(icon) = icon {
		if let Some(icon) = icon.strip_prefix("builtin:") {
			match icon {
				"/icons/fabric.png" => ("fabric", FABRIC).into(),
				"/icons/folia.png" => ("folia", FOLIA).into(),
				"/icons/forge.png" => ("forge", FORGE).into(),
				"/icons/minecraft.png" => ("minecraft", MINECRAFT).into(),
				"/icons/neoforged.png" | "icons/neoforged.png" => ("neoforged", NEOFORGED).into(),
				"/icons/paper.png" => ("paper", PAPER).into(),
				"/icons/quilt.png" => ("quilt", QUILT).into(),
				"/icons/sponge.png" => ("sponge", SPONGE).into(),
				_ => default.into(),
			}
		} else if icon.starts_with("http") {
			Url::parse(icon)
				.unwrap_or(Url::parse("https://example.com").unwrap())
				.into()
		} else {
			let p = icon.strip_prefix("file://").unwrap_or(icon);
			let pb = PathBuf::from(p);
			if pb.exists() {
				pb.into()
			} else {
				default.into()
			}
		}
	} else {
		default.into()
	}
}

pub fn get_loader_icon(loader: &Loader) -> ImageViewer {
	let default = ("default-instance", DEFAULT_INSTANCE);
	let source: ImageSource = match loader {
		Loader::Vanilla => ("minecraft", MINECRAFT).into(),
		Loader::Fabric => ("fabric", FABRIC).into(),
		Loader::Folia => ("folia", FOLIA).into(),
		Loader::Forge => ("forge", FORGE).into(),
		Loader::NeoForged => ("neoforged", NEOFORGED).into(),
		Loader::Paper => ("paper", PAPER).into(),
		Loader::Quilt => ("quilt", QUILT).into(),
		Loader::Sponge | Loader::SpongeForge => ("sponge", SPONGE).into(),
		_ => default.into(),
	};
	ImageViewer::new(source)
		.width(Size::px(16.0))
		.height(Size::px(16.0))
}

pub fn get_package_kind_icon(kind: PackageKind) -> &'static str {
	match kind {
		PackageKind::Mod => "box_plus",
		PackageKind::ResourcePack => "palette",
		PackageKind::Datapack => "curly_braces",
		PackageKind::Plugin => "jigsaw",
		PackageKind::Shader => "sun",
		PackageKind::Modpack => "honeycomb",
		PackageKind::Bundle => "folder",
	}
}

pub async fn pick_image_file() -> Option<PathBuf> {
	#[cfg(target_os = "linux")]
	{
		if which::which("kdialog").is_ok() {
			let is_kde = std::env::var("XDG_CURRENT_DESKTOP")
				.map(|s| s.to_uppercase().contains("KDE"))
				.unwrap_or(false);
			if is_kde {
				let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
				if let Ok(output) = tokio::process::Command::new("kdialog")
					.arg("--title")
					.arg("Выберите изображение для иконки")
					.arg("--getopenfilename")
					.arg(home)
					.arg("*.png *.jpg *.jpeg *.webp *.gif *.bmp *.ico *.svg|Изображения (*.png *.jpg *.jpeg *.webp *.gif *.bmp *.ico *.svg)")
					.output()
					.await
				{
					if output.status.success() {
						let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
						if !path_str.is_empty() {
							let pb = PathBuf::from(path_str);
							if pb.exists() {
								return Some(pb);
							}
						}
					}
				}
			}
		}

		if which::which("zenity").is_ok() {
			if let Ok(output) = tokio::process::Command::new("zenity")
				.arg("--file-selection")
				.arg("--title=Выберите изображение для иконки")
				.arg(
					"--file-filter=Изображения | *.png *.jpg *.jpeg *.webp *.gif *.bmp *.ico *.svg",
				)
				.output()
				.await
			{
				if output.status.success() {
					let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
					if !path_str.is_empty() {
						let pb = PathBuf::from(path_str);
						if pb.exists() {
							return Some(pb);
						}
					}
				}
			}
		}
	}

	tokio::task::spawn_blocking(|| {
		rfd::FileDialog::new()
			.set_title("Выберите изображение для иконки")
			.add_filter(
				"Изображения",
				&[
					"png", "jpg", "jpeg", "webp", "gif", "svg", "bmp", "ico", "avif",
				],
			)
			.pick_file()
	})
	.await
	.ok()
	.flatten()
}
