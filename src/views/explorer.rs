use std::path::{PathBuf};

use iced::widget::scrollable::{Id, RelativeOffset};
use iced::widget::{scrollable, Column, text, row, column, container, Space};
use iced::{event, window, Color, Element, Event, Length, Size, Subscription, Task};

use crate::widgets::fileitem::{FileItem, FileData};

#[derive(Debug, Clone)]
pub enum Message {
    HighlightFile(FileData),
    SelectFile(FileData),
    OpenFile(FileData),

    LoadTree(Vec<FileData>),
    EventOccurred(Event),
    WindowResized(Size),
}

pub struct Explorer {
    current_path: PathBuf,
    tree: Option<Vec<FileData>>,

    highlighted_file: Option<String>,

    width: Option<f32>,
    height: Option<f32>,
}

fn load_tree(path: String) -> Task<Message> {
    Task::perform(
        async {
            let mut files = std::fs::read_dir(path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(FileData::new)
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
                current_path: PathBuf::from("C:\\"),
                tree: None,
                highlighted_file: None,
                width: None,
                height: None,
            },
            Task::batch(vec![
                window::get_oldest().and_then(window::get_size).map(|size| {
                    Message::WindowResized(size)
                }),
                load_tree("C:\\".to_owned()),
            ])
        )
    }

    pub fn open() -> (Self, Task<Message>) {
        Self::new()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectFile(item) => {
                if !item.is_dir {
                    Task::none()
                } else {
                    if item.name == ".." {
                        self.current_path = self.current_path.parent().unwrap_or(&PathBuf::from("C:\\")).to_path_buf();
                    } else {
                        self.current_path = item.path.clone();
                    }
                    Task::batch(vec![
                        load_tree(item.path.to_string_lossy().to_string()),
                        scrollable::snap_to(Id::new("explorer"), RelativeOffset { x: 0.0, y: 0.0 }),
                    ])
                }
            }
            Message::LoadTree(tree) => {
                self.tree = Some(tree);
                Task::none()
            }
            Message::HighlightFile(item) => {
                self.highlighted_file = Some(item.name.clone());
                Task::none()
            }
            Message::OpenFile(item) => {
                println!("Opening file: {:?}", item);
                Task::none()
            }
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(window::Event::CloseRequested) => {
                        println!("Window close requested");
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

        row![
            self.sidebar(),
            column![
                container(self.header())
                    .width(self.width.unwrap_or(200.0) - 200.0),
                scrollable(col.padding(5))
                    .width(self.width.unwrap_or(200.0) - 200.0)
                    .id(Id::new("explorer"))
                    // .height(Length::Fill)
            ]
                .spacing(5),
        ]
            .padding(10)
            .spacing(10)
            .into()
    }

    pub fn sidebar(&self) -> Element<Message> {
        let sidebar = column![
            text("Sidebar Item 1"),
            text("Sidebar Item 2"),
            text("Sidebar Item 3"),
        ]
            .spacing(10);

        scrollable(sidebar)
            .width(200)
            .height(Length::Fill)
            .into()
    }

    pub fn header(&self) -> Element<Message> {
        column![
            text(self.current_path.to_string_lossy().to_string()),
            row![
                Space::with_width(17),
                text("Name")
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                    .width(Length::FillPortion(4))
                    .size(14),
                text("Size")
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                    .width(Length::FillPortion(1))
                    .size(14),
                text("Created at")
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                    .width(Length::FillPortion(2))
                    .size(14),
                text("Last Modified")
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                    .width(Length::FillPortion(2))
                    .size(14),
            ]
                .spacing(5)
                .padding([0, 5])
        ]
            .spacing(5)
            .into()
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