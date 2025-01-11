// use pex::Instruction;

mod pex;

use iced::{
	Alignment::Center,
	Border, Color,
	Length::{self, Fill},
	Task, Theme, color,
	widget::{
		button, center, column, container, row, scrollable, text, text_editor,
	},
};
use nanoserde::SerRon;
struct EditorState {
	content: iced::widget::text_editor::Content,
	visible: bool,
}

struct Tab {
	path: std::path::PathBuf,
	content: Vec<u8>,

	rons: Vec<Vec<Vec<String>>>,

	// object_idx -> state_idx -> function_idx -> (Content, visibility)
	editors: Vec<Vec<Vec<EditorState>>>,

	// Todo: Make this lazily parsed, so Option<Pex>
	pex: pex::Pex,
}

#[derive(Default)]
struct App {
	theme: Theme,

	active: usize,
	active_section: usize,
	active_object: usize,
	tabs: Vec<Tab>,
}

#[derive(Debug, Clone)]
enum Message {
	Open,
	Editor(usize, usize, iced::widget::text_editor::Action),
	SwitchTab(usize),
	SwitchSection(usize),
	SwitchObject(usize),
	ToggleEditor(usize, usize),
}

impl App {
	const BG_DARKER: Color = color!(20, 20, 20);
	const BG: Color = color!(30, 30, 30);
	const BG_GLOW: Color = color!(50, 50, 50);
	const BG_GLOW_BRIGHT: Color = color!(70, 70, 70);

	const TEXT: Color = Color::WHITE;

	const BORDER: Border = Border {
		width: 1.0,
		color: Self::BG_GLOW,
		radius: iced::border::Radius {
			top_left: 0.0,
			top_right: 0.0,
			bottom_left: 0.0,
			bottom_right: 0.0,
		},
	};

	pub fn style_button(active: bool) -> iced::widget::button::Style {
		iced::widget::button::Style {
			background: Some(if active { Self::BG_GLOW } else { Self::BG }.into()),
			text_color: Self::TEXT,
			border: Self::BORDER,
			..Default::default()
		}
	}

	pub fn view(&self) -> iced::Element<Message> {
		if self.tabs.is_empty() {
			return column![center(
				button("Open a file..")
					.style(|_, _| Self::style_button(false))
					.on_press(Message::Open)
					.padding(8)
			)]
			.width(Fill)
			.height(Fill)
			.into();
		}

		let tabs = row![].height(30).width(Fill);

		let tabs = tabs.push(
			scrollable(row(self
				.tabs
				.iter()
				.enumerate()
				.map(|(i, tab)| {
					let text = text(tab.path.file_name().unwrap().to_string_lossy())
						.wrapping(text::Wrapping::None);

					button(text)
						.style(move |_, _| Self::style_button(i == self.active))
						.on_press_maybe(if i != self.active {
							Some(Message::SwitchTab(i))
						} else {
							None
						})
						.into()
				})
				.collect::<Vec<_>>()))
			.direction(iced::widget::scrollable::Direction::Horizontal(
				iced::widget::scrollable::Scrollbar::new(),
			))
			.width(Length::Fill)
			.style(|_, _| iced::widget::scrollable::Style {
				container: iced::widget::container::Style {
					background: Some(Self::BG_DARKER.into()),
					..Default::default()
				},
				vertical_rail: iced::widget::scrollable::Rail {
					background: None,
					scroller: iced::widget::scrollable::Scroller {
						color: Color::WHITE,
						border: Self::BORDER,
					},
					border: Self::BORDER,
				},
				horizontal_rail: iced::widget::scrollable::Rail {
					background: None,
					scroller: iced::widget::scrollable::Scroller {
						color: Color::TRANSPARENT,
						border: Default::default(),
					},
					border: Default::default(),
				},
				gap: Some(color!(255, 0, 0).into()),
			}),
		);

		let tabs = tabs.push(
			button("+")
				.style(|_, _| Self::style_button(false))
				.on_press(Message::Open),
		);

		let tab = &self.tabs[self.active];

		let overview = column![].width(Fill);
		let mut overview = overview.push(
			scrollable(row![
				button("Objects")
					.on_press(Message::SwitchSection(0))
					.style(|_, _| Self::style_button(self.active_section == 0)),
				button("Strings")
					.on_press(Message::SwitchSection(1))
					.style(|_, _| Self::style_button(self.active_section == 1)),
				button("Main")
					.on_press(Message::SwitchSection(2))
					.style(|_, _| Self::style_button(self.active_section == 2)),
			])
			.direction(iced::widget::scrollable::Direction::Horizontal(
				iced::widget::scrollable::Scrollbar::new(),
			))
			.style(|_, _| iced::widget::scrollable::Style {
				container: iced::widget::container::Style {
					background: Some(Self::BG.into()),
					..Default::default()
				},
				vertical_rail: iced::widget::scrollable::Rail {
					background: None,
					scroller: iced::widget::scrollable::Scroller {
						color: Color::WHITE,
						border: Self::BORDER,
					},
					border: Self::BORDER,
				},
				horizontal_rail: iced::widget::scrollable::Rail {
					background: None,
					scroller: iced::widget::scrollable::Scroller {
						color: Color::TRANSPARENT,
						border: Default::default(),
					},
					border: Default::default(),
				},
				gap: Some(color!(255, 0, 0).into()),
			})
			.width(Fill),
		);

		let editor;

		match self.active_section {
			0 => {
				overview = overview.extend(
					tab.pex
						.objects
						.iter()
						.enumerate()
						.map(|(i, (name_idx, _))| {
							button(text(&tab.pex.stringtable[*name_idx as usize]))
								.style(move |_, _| Self::style_button(self.active_object == i))
								.on_press(Message::SwitchObject(i))
								.width(Fill)
								.into()
						})
						.collect::<Vec<_>>(),
				);

				let (_, obj) = &tab.pex.objects[self.active_object];
				let state =
					obj.states.iter().enumerate().map(|(state_idx, state)| {
						let state_name = &tab.pex.stringtable[state.name_idx as usize];

						let functions = state.functions.iter().enumerate().map(
							|(func_idx, (name_idx, _func))| {
								let name = &tab.pex.stringtable[*name_idx as usize];
								let editor_state =
									&tab.editors[self.active_object][state_idx][func_idx];

								let mut col = column![
									row![
										text(name).size(20),
										button(if editor_state.visible { "<" } else { ">" })
											.on_press(Message::ToggleEditor(state_idx, func_idx))
									]
									.align_y(Center)
									.spacing(8)
								];

								if editor_state.visible {
									col = col.push(container(
										text_editor(&editor_state.content)
											.style(|_, _| iced::widget::text_editor::Style {
												background: Self::BG.into(),
												border: Self::BORDER,
												selection: Self::BG_GLOW,
												value: Self::TEXT,
												icon: Self::TEXT,
												placeholder: Self::TEXT,
											})
											.highlight(
												"rust",
												iced::highlighter::Theme::InspiredGitHub,
											)
											.on_action(move |a| {
												Message::Editor(state_idx, func_idx, a)
											})
											.padding(8),
									));
								}

								col.spacing(5).into()
							},
						);

						column![
							text(format!("State: {state_name}")),
							column(functions).spacing(20)
						]
						.spacing(10)
						.into()
					});

				editor = scrollable(column(state).padding(8));
			}
			1 => {
				editor = scrollable(column(
					tab.pex
						.stringtable
						.iter()
						.enumerate()
						.map(|(i, s)| {
							row![
								container(text(format!("{}", i)))
									.style(|_| {
										iced::widget::container::Style {
											border: Self::BORDER,
											..Default::default()
										}
									})
									.width(Fill)
									.height(Fill),
								container(text(s))
									.style(|_| iced::widget::container::Style {
										border: Self::BORDER,
										..Default::default()
									})
									.width(Fill)
									.height(Fill),
								container(button("x").style(|_, _| Self::style_button(false)))
									.style(|_| iced::widget::container::Style {
										border: Self::BORDER,
										..Default::default()
									})
									.width(Fill)
									.height(Fill)
							]
							.padding(4)
							.height(40)
							.into()
						})
						.collect::<Vec<_>>(),
				));
			}
			2 => {
				editor = scrollable(column![]);
			}
			_ => unreachable!(),
		}

		let editor = editor
			.width(Length::FillPortion(6))
			.id(scrollable::Id::new("editor"));

		let tree = scrollable(overview)
			.width(Length::FillPortion(2))
			.height(Length::Fill)
			.style(|_, _| iced::widget::scrollable::Style {
				container: iced::widget::container::Style {
					border: Self::BORDER,
					..Default::default()
				},
				vertical_rail: iced::widget::scrollable::Rail {
					background: None,
					scroller: iced::widget::scrollable::Scroller {
						color: Color::WHITE,
						border: Default::default(),
					},
					border: Self::BORDER,
				},
				horizontal_rail: iced::widget::scrollable::Rail {
					background: None,
					scroller: iced::widget::scrollable::Scroller {
						color: Color::WHITE,
						border: Default::default(),
					},
					border: Self::BORDER,
				},
				gap: Some(color!(255, 0, 0).into()),
			});

		let body = row![tree, editor];

		container(column![tabs, body])
			.style(|_| iced::widget::container::Style {
				background: Some(Self::BG.into()),
				..Default::default()
			})
			.into()
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::Open => 'blk: {
				let Some(paths) = rfd::FileDialog::new()
					.set_title("Open file to yea")
					.add_filter("PEX Files", &["pex"])
					.pick_files()
				else {
					break 'blk;
				};

				for path in paths {
					let bytes = std::fs::read(&path).expect("Failed to read file");
					let pex = pex::parse(&bytes).expect("This isn't a .pex file");

					let rons = pex
						.objects
						.iter()
						.map(|(_, o)| {
							o.states
								.iter()
								.map(|s| {
									s.functions
										.iter()
										.map(|(_, f)| f.serialize_ron())
										.collect::<Vec<_>>()
								})
								.collect::<Vec<_>>()
						})
						.collect::<Vec<_>>();

					let editors = rons
						.iter()
						.map(|object_ron| {
							object_ron
								.iter()
								.map(|state_ron| {
									state_ron
										.iter()
										.map(|function_ron| EditorState {
											content: iced::widget::text_editor::Content::with_text(
												&function_ron,
											),
											visible: false,
										})
										.collect::<Vec<_>>()
								})
								.collect::<Vec<_>>()
						})
						.collect::<Vec<_>>();

					self.tabs.push(Tab {
						path,
						content: bytes,
						pex,
						rons,
						editors,
					})
				}
			}

			Message::SwitchTab(i) => {
				self.active = i;
			}

			Message::SwitchSection(i) => {
				self.active_section = i;
			}

			Message::SwitchObject(i) => {
				self.active_object = i;
			}

			Message::Editor(state_idx, func_idx, action) => {
				self.tabs[self.active].editors[self.active_object][state_idx][func_idx]
					.content
					.perform(action);
			}

			Message::ToggleEditor(state_idx, func_idx) => {
				self.tabs[self.active].editors[self.active_object][state_idx][func_idx].visible ^=
					true;
			}
		};

		Task::none()
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	iced::application("pexSpy", App::update, App::view)
		.window_size((768.0, 512.0))
		.run()?;

	Ok(())
}
