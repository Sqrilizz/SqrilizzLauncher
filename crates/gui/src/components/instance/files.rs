use std::path::{Path, PathBuf};

use sqrilizz::{core::QuickPlayType, shared::util::open_link};

use crate::{
    components::{
        dialog::{
            modal::{Modal, ModalButton},
            toast::Toast,
        },
        misc::number_indicator,
    },
    ops::{
        instance::FetchInstanceFiles,
        launch::{LaunchInstance, LaunchInstanceParams},
    },
    prelude::*,
    state::FrontState,
    util::Shared,
};

#[derive(PartialEq)]
pub struct InstanceFilesView {
    pub id: String,
}

impl Component for InstanceFilesView {
    fn render(&self) -> impl IntoElement {
        let theme = use_theme();
        let front_state = use_front_state();
        let back_state = use_consume::<BackState>();
        let files_query = use_query(Query::new(
            self.id.clone(),
            FetchInstanceFiles::new(back_state.clone()),
        ));
        let files = files_query.read().state().ok().cloned().unwrap_or_default();

        let previewed_screenshot = use_state::<Option<PathBuf>>(|| None);

        let save_count = files.saves.len();
        let saves: Vec<_> = files
            .saves
            .into_iter()
            .map(|x| {
                Item {
                    icon: x.icon_path.map(|x| ImageSource::Path(PathBuf::from(x))),
                    name: x.name,
                    ty: ItemType::Save,
                    instance_id: self.id.clone(),
                    server_address: None,
                }
            })
            .collect();

        let left_top = rect()
            .width(Size::fill())
            .height(Size::percent(50.0))
            .padding(theme.gap2)
            .spacing(theme.gap2)
            .border(border_bottom(theme.border, theme.panel_border))
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .spacing(theme.gap)
                    .center()
                    .child(icon("minecraft", 16.0))
                    .child("Миры")
                    .child(number_indicator(save_count, &theme)),
            )
            .child(
                ScrollView::new()
                    .width(Size::fill())
                    .height(Size::fill())
                    .spacing(theme.gap)
                    .children(saves),
            );

        let screenshot_count = files.screenshots.len();
        let preview_setter_state = previewed_screenshot;
        let screenshot_items: Vec<_> = files
            .screenshots
            .into_iter()
            .map(|path| {
                let p = path.clone();
                let mut preview_setter = preview_setter_state;
                ScreenshotItem {
                    path,
                    on_click: EventHandler::new(move |_| {
                        preview_setter.set(Some(p.clone()));
                    }),
                }
                .into_element()
            })
            .collect();
        let screenshots = if screenshot_count == 0 {
            rect()
                .expanded()
                .center()
                .child(placeholder("Нет скриншотов", &theme))
                .into_element()
        } else {
            ScrollView::new()
                .expanded()
                .child(grid(2, screenshot_items).gap(theme.gap2))
                .into_element()
        };

        let left_bottom = rect()
            .width(Size::fill())
            .height(Size::percent(50.0))
            .padding(theme.gap2)
            .spacing(theme.gap2)
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .spacing(theme.gap)
                    .center()
                    .child(icon("picture", 16.0))
                    .child("Скриншоты")
                    .child(number_indicator(screenshot_count, &theme)),
            )
            .child(screenshots);

        let left = rect()
            .width(Size::percent(50.0))
            .height(Size::fill())
            .spacing(theme.gap2)
            .border(border_right(theme.border, theme.panel_border))
            .child(left_top)
            .child(left_bottom);

        let server_count = files.servers.len();
        let servers: Vec<_> = files
            .servers
            .into_iter()
            .map(|x| {
                super::servers::ServerCard {
                    icon: x.icon_png.map(|p| ImageSource::from(Bytes::from(p))),
                    name: x.name.unwrap_or_else(|| x.address.clone()),
                    address: x.address,
                    instance_id: self.id.clone(),
                    refresh_id: 0,
                }
                .into_element()
            })
            .collect();
        let servers = ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .spacing(theme.gap2)
            .children(servers);

        let right = rect()
            .width(Size::percent(50.0))
            .height(Size::fill())
            .padding(theme.gap2)
            .spacing(theme.gap2)
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .spacing(theme.gap)
                    .center()
                    .child(icon("server", 16.0))
                    .child("Серверы")
                    .child(number_indicator(server_count, &theme)),
            )
            .child(servers);

        let preview_modal = if let Some(path) = previewed_screenshot.read().clone() {
            let filename = path
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or("Screenshot")
                .to_string();

            let modal_title = if let Some((w, h)) = get_png_dimensions(&path) {
                let meta = std::fs::metadata(&path).ok();
                let sz = meta
                    .map(|m| format!(" • {}", format_filesize(m.len())))
                    .unwrap_or_default();
                format!("{} ({}×{}{})", filename, w, h, sz)
            } else {
                filename.clone()
            };

            let path_copy = path.clone();
            let path_del = path.clone();
            let path_open = path.clone();
            let path_upload = path.clone();
            let mut previewed_screenshot_close = previewed_screenshot;
            let mut previewed_screenshot_del = previewed_screenshot;
            let files_query2 = files_query;
            let front_state2 = front_state.clone();
            let front_state3 = front_state.clone();
            let front_state4 = front_state.clone();

            Some(
                Modal::new(modal_title, "picture".into())
                    .size(1280.0, 820.0)
                    .on_close(move |_| {
                        previewed_screenshot_close.set(None);
                    })
                    .button(ModalButton {
                        title: "Открыть в систему".into(),
                        icon: "popout".into(),
                        on_click: (move |_| {
                            let _ = open::that(&path_open);
                        })
                        .into(),
                        active: false,
                        danger: false,
                    })
                    .button(ModalButton {
                        title: "Скопировать путь".into(),
                        icon: "copy".into(),
                        on_click: (move |_| {
                            let _ = Clipboard::set(path_copy.to_string_lossy().to_string());
                            front_state2
                                .write()
                                .toast(Toast::info("Путь к скриншоту скопирован", None));
                        })
                        .into(),
                        active: false,
                        danger: false,
                    })
                    .button(ModalButton {
                        title: "Загрузить на хостинг".into(),
                        icon: "upload".into(),
                        on_click: (move |_| {
                            let p = path_upload.clone();
                            let fs = front_state3.clone();
                            spawn_forever(async move {
                                upload_screenshot(p, fs).await;
                            });
                        })
                        .into(),
                        active: true,
                        danger: false,
                    })
                    .button(ModalButton {
                        title: "Удалить".into(),
                        icon: "trash".into(),
                        on_click: (move |_| {
                            let _ = std::fs::remove_file(&path_del);
                            files_query2.invalidate();
                            previewed_screenshot_del.set(None);
                            front_state4
                                .write()
                                .toast(Toast::info("Скриншот удален", None));
                        })
                        .into(),
                        active: false,
                        danger: true,
                    })
                    .cancel_button()
                    .maybe_child(true, || {
                        rect()
                            .expanded()
                            .padding(theme.gap)
                            .center()
                            .child(
                                crate::util::gif::AnimatedIconViewer::new(Some(
                                    path.to_string_lossy().to_string(),
                                ))
                                .expanded()
                                .aspect_ratio(AspectRatio::Max)
                                .image_cover(ImageCover::Center)
                                .corner_radius(theme.round),
                            )
                    },
                    ),
            )
        } else {
            None
        };

        rect()
            .expanded()
            .child(rect().expanded().horizontal().child(left).child(right))
            .maybe_child(preview_modal)
    }
}

#[derive(PartialEq)]
enum ItemType {
    Save,
}

#[derive(PartialEq, Clone)]
struct ScreenshotItem {
    path: PathBuf,
    on_click: EventHandler<()>,
}

impl Component for ScreenshotItem {
    fn render(&self) -> impl IntoElement {
        let theme = use_theme();
        let is_hovered = use_state(|| false);

        let filename = self
            .path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("Screenshot")
            .to_string();

        let path_str = self.path.to_string_lossy().to_string();
        let on_click = self.on_click.clone();

        let border_color = if *is_hovered.read() {
            theme.item_select_border
        } else {
            theme.panel_border
        };

        rect()
            .width(Size::fill())
            .height(Size::px(145.0))
            .border(theme.border(border_color))
            .corner_radius(theme.round)
            .background(theme.panel)
            .hover(is_hovered)
            .cursor(CursorIcon::Pointer)
            .clickable()
            .on_press(move |_| on_click.call(()))
            .child(
                crate::util::gif::AnimatedIconViewer::new(Some(path_str))
                    .width(Size::fill())
                    .height(Size::px(112.0))
                    .corner_radius(CornerRadius::new(theme.round, theme.round, 0.0, 0.0))
                    .image_cover(ImageCover::Center),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(32.0))
                    .background(theme.bg)
                    .padding(Gaps::new(0.0, theme.gap, 0.0, theme.gap))
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .child(
                        label()
                            .text(filename)
                            .max_lines(1)
                            .color(theme.fg)
                            .font_size(11.0),
                    )
                    .child(icon("popout", 12.0).color(theme.fg3)),
            )
    }
}

#[derive(PartialEq)]
struct Item {
    icon: Option<ImageSource>,
    name: String,
    ty: ItemType,
    instance_id: String,
    server_address: Option<String>,
}

impl Component for Item {
    fn render(&self) -> impl IntoElement {
        let theme = use_theme();
        let back_state = use_consume::<BackState>();

        let is_hovered = use_state(|| false);

        let launch_mutation = use_mutation(LaunchInstance::new(back_state));
        let instance_id = self.instance_id.clone();
        let name = self.name.clone();
        let launch_button = Button::new()
            .child(icon("play", 16.0))
            .child("Играть")
            .on_press({
                let instance_id = instance_id.clone();
                let name = name.clone();
                move |_| {
                    launch_mutation.mutate(LaunchInstanceParams {
                        id: instance_id.clone(),
                        offline: false,
                        quick_play: QuickPlayType::World {
                            world: name.clone(),
                        },
                    });
                }
            });

        rect()
            .width(Size::fill())
            .height(Size::px(48.0))
            .border(theme.border(theme.panel_border))
            .corner_radius(theme.round)
            .hover(is_hovered)
            .panel_colorway(&theme, *is_hovered.read(), false)
            .cont()
            .cross_align(Alignment::Center)
            .child(icon("minecraft", 20.0))
            .child(segment(self.name.as_str(), 1.0))
            .child(launch_button)
    }
}

fn format_filesize(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} МБ", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{} КБ", bytes / 1024)
    } else {
        format!("{} Б", bytes)
    }
}

async fn upload_screenshot(path: PathBuf, front_state: Shared<FrontState>) {
    front_state
        .write()
        .toast(Toast::info("Загрузка скриншота на хостинг...", None));

    let Ok(bytes) = tokio::fs::read(&path).await else {
        front_state
            .write()
            .toast(Toast::error("Не удалось прочитать файл скриншота", None));
        return;
    };

    let fname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("screenshot.png")
        .to_string();

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let catbox_res: Option<String> = async {
        let part = reqwest::multipart::Part::bytes(bytes.clone())
            .file_name(fname.clone())
            .mime_str("image/png")
            .ok()?;
        let form = reqwest::multipart::Form::new()
            .text("reqtype", "fileupload")
            .part("fileToUpload", part);

        let resp = client
            .post("https://catbox.moe/user/api.php")
            .multipart(form)
            .send()
            .await
            .ok()?;
        if resp.status().is_success() {
            let text = resp.text().await.ok()?;
            let url = text.trim().to_string();
            if url.starts_with("http") {
                return Some(url);
            }
        }
        None
    }
    .await;

    let final_url = if let Some(url) = catbox_res {
        Some(url)
    } else {
        let uguu_res: Option<String> = async {
            let part = reqwest::multipart::Part::bytes(bytes)
                .file_name(fname)
                .mime_str("image/png")
                .ok()?;
            let form = reqwest::multipart::Form::new().part("files[]", part);

            let resp = client
                .post("https://uguu.se/upload.php")
                .multipart(form)
                .send()
                .await
                .ok()?;
            if resp.status().is_success() {
                let json: serde_json::Value = resp.json().await.ok()?;
                let url = json.get("files")?.get(0)?.get("url")?.as_str()?;
                return Some(url.to_string());
            }
            None
        }
        .await;
        uguu_res
    };

    if let Some(url) = final_url {
        let _ = Clipboard::set(url.clone());
        let _ = open_link(&url);
        front_state
            .write()
            .toast(Toast::success("Скриншот загружен (ссылка скопирована)"));
    } else {
        front_state
            .write()
            .toast(Toast::error("Ошибка загрузки скриншота на хостинг", None));
    }
}

fn get_png_dimensions(path: &Path) -> Option<(u32, u32)> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 24];
    use std::io::Read;
    file.read_exact(&mut buf).ok()?;
    if &buf[0..8] == b"\x89PNG\r\n\x1a\n" && &buf[12..16] == b"IHDR" {
        let width = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let height = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
        Some((width, height))
    } else {
        None
    }
}

fn segment(text: &str, _: f32) -> impl IntoElement {
    rect().child(text)
}