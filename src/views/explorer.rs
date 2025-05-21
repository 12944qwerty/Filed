use std::path::{PathBuf};

use iced::advanced::{mouse, widget::operation};
use iced::widget::button::Style;
use iced::widget::scrollable::{Id, RelativeOffset};
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input, Column, Space};
use iced::{event, window, Color, Element, Event, Length, Padding, Pixels, Size, Subscription, Task};
use iced_runtime::{Action, task};

use crate::components::fileitem::{FileItem, FileData};
use crate::platform::Platform;

#[derive(Debug, Clone)]
pub enum Message {
    SelectFile(FileData),
    OpenFile(FileData),
    History(bool),

    LoadTree(Vec<FileData>),
    EventOccurred(Event),
    WindowResized(Size),

    ClickedOn(container::Id),

    AddressbarChanged(String),
    DirChanged,
}

pub struct Explorer {
    current_path: PathBuf,
    tree: Option<Vec<FileData>>,

    highlighted_file: Option<String>,

    width: Option<f32>,
    height: Option<f32>,

    history: Vec<PathBuf>,
    history_index: usize,

    addressbar_focused: bool,
    addressbar_content: String,
}

fn load_tree(path: PathBuf) -> Task<Message> {
    Task::perform(
        async {
            let mut files = std::fs::read_dir(path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|e| { FileData::new(e.path()) })
                .collect::<Vec<_>>();
            files
                .sort_by_key(|i| !i.is_dir);

            files
        },
        Message::LoadTree
    )
}

impl Explorer {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_path: Platform::home_dir(),
                tree: None,
                highlighted_file: None,
                width: None,
                height: None,
                history: vec![Platform::home_dir()],
                history_index: 0,
                addressbar_focused: false,
                addressbar_content: "".to_string(),
            },
            Task::batch(vec![
                window::get_oldest().and_then(window::get_size).map(|size| {
                    Message::WindowResized(size)
                }),
                load_tree(Platform::home_dir()),
            ])
        )
    }

    pub fn open() -> (Self, Task<Message>) {
        Self::new()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile(item) => {
                if !item.is_dir {
                    Task::none()
                } else if self.current_path == item.path {
                    Task::none()
                } else {
                    if item.name == ".." {
                        self.current_path = self.current_path.parent().unwrap_or(&PathBuf::from("C:\\")).to_path_buf();
                    } else {
                        self.current_path = item.path.clone();
                    }
                    self.history.truncate(self.history_index + 1);
                    self.history.push(self.current_path.clone());
                    self.history_index += 1;

                    Task::batch(vec![
                        load_tree(item.path),
                        scrollable::snap_to(Id::new("explorer"), RelativeOffset { x: 0.0, y: 0.0 }),
                    ])
                }
            }
            Message::History(forward) => {
                if forward {
                    if self.history_index < self.history.len() - 1 {
                        self.history_index += 1;
                        self.current_path = self.history[self.history_index].clone();
                    }
                } else {
                    if self.history_index > 0 {
                        self.history_index -= 1;
                        self.current_path = self.history[self.history_index].clone();
                    }
                }
                Task::batch(vec![
                    load_tree(self.current_path.clone()),
                    scrollable::snap_to(Id::new("explorer"), RelativeOffset { x: 0.0, y: 0.0 }),
                ])
            }
            Message::LoadTree(tree) => {
                self.tree = Some(tree);
                Task::none()
            }
            Message::SelectFile(item) => {
                self.addressbar_focused = false;
                self.highlighted_file = Some(item.name.clone());
                Task::none()
            }
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(window::Event::CloseRequested) => {
                        println!("Window close requested");
                        Task::none()
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        println!("Clicked somewhere");
                        self.addressbar_focused = false;
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
            Message::WindowResized(size) => {
                self.width = Some(size.width);
                self.height = Some(size.height);
                Task::none()
            }
            Message::ClickedOn(id) => {
                if id == "addressbar".into() {
                    self.addressbar_content = self.current_path.to_string_lossy().to_string();
                    self.addressbar_focused = true;
                    return text_input::focus("addressbar_inp").then(|_: Task<Message>| Task::from(text_input::select_all("addressbar_inp")));
                }
                Task::none()
            }
            Message::AddressbarChanged(content) => {
                // println!("Addressbar changed: {}", content.clone());
                self.addressbar_content = content;
                Task::none()
            }
            Message::DirChanged => {
                if self.addressbar_content.is_empty() {
                    return task::effect(Action::widget(operation::focusable::unfocus()));
                }
                let path = PathBuf::from(self.addressbar_content.clone());
                if path.exists() {
                    self.current_path = path;
                    self.history.truncate(self.history_index + 1);
                    self.history.push(self.current_path.clone());
                    self.history_index += 1;
                    self.addressbar_focused = false;
                    return Task::batch(vec![
                        load_tree(self.current_path.clone()),
                        task::effect(Action::widget(operation::focusable::unfocus())),
                    ]);
                }
                task::effect(Action::widget(operation::focusable::unfocus()))
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut col: Column<'_, Message> = Column::new().spacing(5);

        let mut tree = self.tree.clone().unwrap_or(vec![]);
        tree.insert(0, FileData::parent(self.current_path.clone()));


        for data in tree {
            col = col.push(
                FileItem::from(data.clone())
                    .is_highlighted(self.highlighted_file.clone().unwrap_or("".to_string()) == data.name)
                    .on_select(Box::new(Message::SelectFile))
                    .on_open(Box::new(Message::OpenFile))
            );
        }

        column![
            self.header(),
            row![
                self.sidebar(),
                column![
                    container(self.tableheader())
                        .width(self.width.unwrap_or(200.0) - 200.0),
                    scrollable(col.padding(5))
                        .width(self.width.unwrap_or(200.0) - 200.0)
                        .id("explorer")
                        // .height(Length::Fill)
                ]
            ]
                .spacing(5),
        ]
            .padding(10)
            .spacing(10)
            .into()
    }

    pub fn sidebar(&self) -> Element<Message> {
        let mut sidebar = Column::new()
            .spacing(5);

        for item in Platform::special_dirs() {
            sidebar = sidebar.push(
                FileItem::new(item.clone())
                    .on_select(Box::new(Message::SelectFile))
                    .on_open(Box::new(Message::OpenFile))
                    .sidebar()
            );
        }

        scrollable(sidebar)
            .width(200)
            .height(Length::Fill)
            .into()
    }

    pub fn tableheader(&self) -> Element<Message> {
        row![
            Space::with_width(17),
            text("Name")
                .width(Length::FillPortion(4))
                .size(14),
            text("Size")
                .width(Length::FillPortion(1))
                .size(14),
            text("Created at")
                .width(Length::FillPortion(2))
                .size(14),
            text("Last Modified")
                .width(Length::FillPortion(2))
                .size(14),
        ]
            .spacing(5)
            .padding([0, 5])
            .into()
    }

    pub fn header(&self) -> Element<Message> {
        container(
            row![
                container(
                    button(
                        text("<").size(20).height(Length::Fill)
                    )
                        .padding(Padding::new(0.0).top(1))
                        .style(button::text)
                        .on_press(Message::History(false))
                )
                    .padding(0)
                    .width(20),
                container(
                    button(
                        text(">").size(20).height(Length::Fill)
                    )
                        .padding(Padding::new(0.0).top(1))
                        .style(button::text)
                        .on_press(Message::History(true))
                )
                    .padding(0)
                    .width(20),
                self.addressbar(),
                // todo search
            ]
                .spacing(5)
                .padding(5)
                .height(Length::Fill)
        )
            .width(self.width.unwrap_or(200.0))
            .height(40)
            .into()
    }

    pub fn addressbar(&self) -> Element<Message> {
        if !self.addressbar_focused {
            container(
                button(
                    text(self.current_path.to_string_lossy().to_string()).width(Length::Fill)
                )
                    .padding(Padding::default().top(4))
                    .style(button::text)
                    .on_press(Message::ClickedOn("addressbar".into()))
                    .width(Length::Fill)
            )
                .width(Length::Fill)
                .id("addressbar")
                .into()
        } else {
            container(
                text_input(Platform::home_dir().to_string_lossy().as_ref(), &self.addressbar_content)
                    .on_input(Message::AddressbarChanged)
                    .on_submit(Message::DirChanged)
                    .id("addressbar_inp")
            )
                .width(Length::Fill)
                .into()
        }
    }

    pub fn title(&self) -> String {
        format!("Filed - {}", self.current_path.to_string_lossy())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
            event::listen().map(|event| Message::EventOccurred(event)),
        ])
    }
}
