use std::{path::Path, rc::Rc};

use sqrilizz::config_crate::template::TemplateConfig;

use crate::{
	components::input::{Derivable, derived_value_owned, select::Selected, text::TextInput},
	ops::plugin_results::FetchJavaTypes,
	pages::config::ConfigState,
	prelude::*,
	util::PtrEq,
};

#[derive(PartialEq)]
pub struct LaunchConfigPage {
	pub config_state: ConfigState,
	pub parent_configs: PtrEq<[TemplateConfig]>,
}

impl Component for LaunchConfigPage {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();

		let contents = rect()
			.width(Size::fill())
			.spacing(theme.gap3)
			.child(JavaSelector {
				java: self.config_state.java,
				parent_configs: self.parent_configs.clone(),
			})
			.child(MemorySelector {
				memory: self.config_state.memory,
				parent_configs: self.parent_configs.clone(),
			})
			.child(JvmArgsSelector {
				jvm_args: self.config_state.jvm_args,
				parent_configs: self.parent_configs.clone(),
			});

		rect()
			.expanded()
			.padding(theme.gap3)
			.child(
				ScrollView::new()
					.expanded()
					.child(contents),
			)
	}
}

#[derive(PartialEq)]
struct JavaSelector {
	java: State<Option<String>>,
	parent_configs: PtrEq<[TemplateConfig]>,
}

#[derive(PartialEq, Clone)]
enum JavaSelected {
	None,
	Standard(String),
	Custom,
}

impl Component for JavaSelector {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let types_query = use_query(Query::new((), FetchJavaTypes::new(back_state.clone())));

		let types = types_query.read().state().ok().cloned().unwrap_or_default();

		let all_options = types
			.iter()
			.map(|x| x.id.as_str())
			.chain(["auto", "system", "adoptium"])
			.collect::<Vec<_>>();

		let is_custom_mode = use_state(|| match &*self.java.read() {
			Some(s) => !s.is_empty() && !all_options.contains(&s.as_str()),
			None => false,
		});

		let is_custom = *is_custom_mode.read();
		let selected = if is_custom {
			JavaSelected::Custom
		} else {
			match &*self.java.read() {
				Some(selected) => {
					if all_options.contains(&selected.as_str()) {
						JavaSelected::Standard(selected.clone())
					} else {
						JavaSelected::None
					}
				}
				None => JavaSelected::None,
			}
		};

		let derived = derived_value_owned(self.java.read().clone(), &self.parent_configs.0, |x| {
			x.instance.launch.java.clone()
		});
		let derived_selected = match &derived {
			Some(selected) => {
				if all_options.contains(&selected.as_str()) {
					Some(JavaSelected::Standard(selected.clone()))
				} else if !selected.is_empty() {
					Some(JavaSelected::Custom)
				} else {
					None
				}
			}
			None => None,
		};

		let path = use_state(|| self.java.read().clone().unwrap_or_default());

		let java_state = self.java;
		let is_custom_state = is_custom_mode;
		let path_state = path;
		let selector = Dropdown::new(
			Selected::Single(selected),
			Rc::new(move |selected| {
				match selected.single() {
					JavaSelected::None => {
						is_custom_state.clone().set(false);
						java_state.clone().set(None);
					}
					JavaSelected::Standard(kind) => {
						is_custom_state.clone().set(false);
						java_state.clone().set(Some(kind));
					}
					JavaSelected::Custom => {
						is_custom_state.clone().set(true);
						let p = path_state.read().clone();
						if !p.is_empty() {
							java_state.clone().set(Some(p));
						}
					}
				}
			}),
		)
		.panel_colorway()
		.derived(derived_selected)
		.child(SelectOption::new(
			JavaSelected::None,
			"По умолчанию (Inherit)",
			Some("diagram"),
		))
		.child(
			SelectOption::new(JavaSelected::Standard("auto".into()), "Авто (Рекомендуется)", Some("star"))
				.tip("Автоматически подобрать лучшую версию Java под версию игры"),
		)
		.child(
			SelectOption::new(
				JavaSelected::Standard("system".into()),
				"Системная Java",
				Some("gear"),
			)
			.tip("Использовать Java из системы"),
		)
		.child(
			SelectOption::new(
				JavaSelected::Standard("adoptium".into()),
				"Adoptium Java",
				Some("box"),
			)
			.tip("Использовать сборку Adoptium"),
		)
		.children(
			types
				.into_iter()
				.map(|x| SelectOption::new(JavaSelected::Standard(x.id), &x.name, Some("box"))),
		)
		.child(SelectOption::new(
			JavaSelected::Custom,
			"Указать путь вручную",
			Some("folder"),
		));
		let selector = field("Версия Java", "play", &theme, selector).tip(
			&front_state,
			"Версия или среда выполнения Java для данного инстанса",
		);

		let mut java_writer = self.java;
		use_side_effect(move || {
			if *is_custom_mode.read() {
				let p = path.read().clone();
				if !p.is_empty() {
					java_writer.set_if_modified(Some(p));
				}
			}
		});

		let exists = Path::new(&*path.read()).exists();
		let custom_input = TextInput::new(path)
			.derived(derived)
			.maybe_input_error(!exists && !path.read().is_empty(), "Путь не существует")
			.placeholder("Введите путь к Java...");
		let custom_input = field("Путь к Java", "folder", &theme, custom_input).tip(
			&front_state,
			"Путь к папке или исполняемому файлу Java",
		);

		rect()
			.width(Size::fill())
			.child(selector)
			.maybe(is_custom, |this| this.child(custom_input))
	}
}

#[derive(PartialEq)]
struct MemorySelector {
	memory: State<Option<String>>,
	parent_configs: PtrEq<[TemplateConfig]>,
}

#[derive(PartialEq, Clone)]
enum MemorySelected {
	None,
	Standard(String),
	Custom,
}

impl Component for MemorySelector {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

		let standard_options = ["2G", "4G", "6G", "8G", "10G", "12G", "16G"];

		let is_custom_mode = use_state(|| match &*self.memory.read() {
			Some(s) => !s.trim().is_empty() && !standard_options.contains(&s.as_str()),
			None => false,
		});

		let is_custom = *is_custom_mode.read();
		let selected = if is_custom {
			MemorySelected::Custom
		} else {
			match &*self.memory.read() {
				Some(s) => {
					if standard_options.contains(&s.as_str()) {
						MemorySelected::Standard(s.clone())
					} else {
						MemorySelected::None
					}
				}
				None => MemorySelected::None,
			}
		};

		let derived = derived_value_owned(self.memory.read().clone(), &self.parent_configs.0, |x| {
			match &x.instance.launch.memory {
				sqrilizz::config_crate::instance::LaunchMemory::Single(s) => Some(s.clone()),
				sqrilizz::config_crate::instance::LaunchMemory::Both { max, .. } => Some(max.clone()),
				_ => None,
			}
		});
		let derived_selected = match &derived {
			Some(s) => {
				if standard_options.contains(&s.as_str()) {
					Some(MemorySelected::Standard(s.clone()))
				} else {
					Some(MemorySelected::Custom)
				}
			}
			None => None,
		};

		let custom_val = use_state(|| self.memory.read().clone().unwrap_or_else(|| "6G".to_string()));

		let memory_state = self.memory;
		let is_custom_state = is_custom_mode;
		let custom_val_state = custom_val;
		let selector = Dropdown::new(
			Selected::Single(selected),
			Rc::new(move |selected| {
				match selected.single() {
					MemorySelected::None => {
						is_custom_state.clone().set(false);
						memory_state.clone().set(None);
					}
					MemorySelected::Standard(ram) => {
						is_custom_state.clone().set(false);
						custom_val_state.clone().set(ram.clone());
						memory_state.clone().set(Some(ram));
					}
					MemorySelected::Custom => {
						is_custom_state.clone().set(true);
						let val = custom_val_state.read().clone();
						let target = if val.trim().is_empty() {
							"6G".to_string()
						} else {
							val
						};
						custom_val_state.clone().set(target.clone());
						memory_state.clone().set(Some(target));
					}
				}
			}),
		)
		.panel_colorway()
		.derived(derived_selected)
		.child(SelectOption::new(
			MemorySelected::None,
			"По умолчанию (Inherit)",
			Some("diagram"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("2G".into()),
			"2 GB (Минимальный)",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("4G".into()),
			"4 GB (Стандартный)",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("6G".into()),
			"6 GB (Рекомендуется для сборок)",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("8G".into()),
			"8 GB (Тяжелые сборки)",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("10G".into()),
			"10 GB",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("12G".into()),
			"12 GB",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Standard("16G".into()),
			"16 GB",
			Some("speed"),
		))
		.child(SelectOption::new(
			MemorySelected::Custom,
			"Своё значение",
			Some("edit"),
		));

		let selector = field("Выделение памяти (ОЗУ / RAM)", "speed", &theme, selector).tip(
			&front_state,
			"Максимальный объем выделяемой оперативной памяти (например: 4G, 6G, 8G)",
		);

		let mut memory_writer = self.memory;
		let custom_val_reader = custom_val;
		use_side_effect(move || {
			if *is_custom_mode.read() {
				let val = custom_val_reader.read().clone();
				if !val.trim().is_empty() {
					memory_writer.set_if_modified(Some(val.trim().to_string()));
				}
			}
		});

		let mut memory_writer2 = self.memory;
		let custom_input = TextInput::new(custom_val)
			.derived(derived)
			.placeholder("например: 6G или 6144M")
			.on_change(move |val: String| {
				if !val.trim().is_empty() {
					memory_writer2.set_if_modified(Some(val.trim().to_string()));
				}
			});
		let custom_input = field("Свой размер памяти", "edit", &theme, custom_input).tip(
			&front_state,
			"Укажите размер в формате 6G, 8192M и т.д.",
		);

		rect()
			.width(Size::fill())
			.child(selector)
			.maybe(is_custom, |this| this.child(custom_input))
	}
}

#[derive(PartialEq)]
struct JvmArgsSelector {
	jvm_args: State<Option<String>>,
	parent_configs: PtrEq<[TemplateConfig]>,
}

#[derive(PartialEq, Clone)]
enum JvmArgsPreset {
	None,
	Aikar,
	Shenandoah,
	Zgc,
	Custom,
}

impl Component for JvmArgsSelector {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();

		const AIKAR: &str = "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1";
		const SHENANDOAH: &str = "-XX:+UseShenandoahGC -XX:+UnlockExperimentalVMOptions -XX:ShenandoahGCHeuristics=adaptive";
		const ZGC: &str = "-XX:+UseZGC -XX:+ZGenerational";

		let current_str = self.jvm_args.read().clone().unwrap_or_default();
		let selected = if current_str.trim().is_empty() {
			JvmArgsPreset::None
		} else if current_str == AIKAR {
			JvmArgsPreset::Aikar
		} else if current_str == SHENANDOAH {
			JvmArgsPreset::Shenandoah
		} else if current_str == ZGC {
			JvmArgsPreset::Zgc
		} else {
			JvmArgsPreset::Custom
		};

		let derived = derived_value_owned(self.jvm_args.read().clone(), &self.parent_configs.0, |x| {
			match &x.instance.launch.args.jvm {
				sqrilizz::config_crate::instance::Args::List(l) if !l.is_empty() => Some(l.join(" ")),
				sqrilizz::config_crate::instance::Args::String(s) if !s.is_empty() => Some(s.clone()),
				_ => None,
			}
		});

		let jvm_args2 = self.jvm_args;
		let selector = Dropdown::new(
			Selected::Single(selected.clone()),
			Rc::new(move |selected| {
				jvm_args2.clone().set(match selected.single() {
					JvmArgsPreset::None => None,
					JvmArgsPreset::Aikar => Some(AIKAR.to_string()),
					JvmArgsPreset::Shenandoah => Some(SHENANDOAH.to_string()),
					JvmArgsPreset::Zgc => Some(ZGC.to_string()),
					JvmArgsPreset::Custom => Some(String::new()),
				});
			}),
		)
		.panel_colorway()
		.child(SelectOption::new(
			JvmArgsPreset::None,
			"По умолчанию / Без флагов",
			Some("diagram"),
		))
		.child(SelectOption::new(
			JvmArgsPreset::Aikar,
			"Флаги Айкара (Оптимизация G1GC)",
			Some("lightning"),
		))
		.child(SelectOption::new(
			JvmArgsPreset::Shenandoah,
			"Shenandoah GC (Сверхнизкие задержки)",
			Some("feather"),
		))
		.child(SelectOption::new(
			JvmArgsPreset::Zgc,
			"Generational ZGC (Java 21+)",
			Some("rocket"),
		))
		.child(SelectOption::new(
			JvmArgsPreset::Custom,
			"Свои аргументы",
			Some("edit"),
		));

		let selector = field("Флаги сборщика мусора (GC) и JVM", "gear", &theme, selector).tip(
			&front_state,
			"Готовые пресеты аргументов JVM для снижения микрофризов и плавного FPS",
		);

		let custom_input_val = use_state(String::new);
		use_side_effect({
			let jvm_args = self.jvm_args;
			let mut custom_input_val = custom_input_val;
			move || {
				if let Some(s) = &*jvm_args.read() {
					custom_input_val.set_if_modified(s.clone());
				}
			}
		});
		use_side_effect({
			let mut jvm_args = self.jvm_args;
			let custom_input_val = custom_input_val;
			move || {
				let val = custom_input_val.read().clone();
				if val.trim().is_empty() {
					jvm_args.set_if_modified(None);
				} else {
					jvm_args.set_if_modified(Some(val));
				}
			}
		});

		let custom_input = TextInput::new(custom_input_val)
			.derived(derived)
			.placeholder("-XX:+UseG1GC ...");
		let custom_input = field("Аргументы JVM", "edit", &theme, custom_input).tip(
			&front_state,
			"Дополнительные флаги JVM через пробел",
		);

		rect()
			.width(Size::fill())
			.child(selector)
			.child(custom_input)
	}
}
