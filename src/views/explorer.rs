use std::path::{PathBuf};
use std::time::{SystemTime};

use iced::widget::scrollable::{Id, RelativeOffset};
use iced::widget::{column, container, mouse_area, row, scrollable, text, Column, Image, Space};
use iced::{event, window, Color, Element, Event, Length, Padding, Size, Subscription, Task};

use crate::utils::{file_type_from_extension, image_from_type, readable_size, readable_time};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum FileType {
    Directory,

    File,
    Image,
    Video,
    Audio,
    Document,

    Unknown,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub last_modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub file_type: Option<FileType>,
}

#[derive(Debug, Clone)]
pub enum Message {
    HighlightFile(FileItem),
    SelectFile(FileItem),
    OpenFile(FileItem),

    LoadTree(Vec<FileItem>),
    EventOccurred(Event),
    WindowResized(Size),
}

pub struct Explorer {
    current_path: PathBuf,
    tree: Option<Vec<FileItem>>,

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
                .map(|entry| {
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    let is_dir = path.is_dir();
                    let metadata = path.metadata().unwrap();
                    let created = metadata.created().ok();
                    let last_modified = metadata.modified().ok();
                    let size = if is_dir { None } else { Some(path.metadata().unwrap().len()) };
                    FileItem {
                        name,
                        path: path.clone(),
                        is_dir,
                        size,
                        last_modified,
                        created,
                        file_type: if is_dir {
                            Some(FileType::Directory)
                        } else {
                            Some(file_type_from_extension(path.extension().and_then(|s| s.to_str()).unwrap_or("")))
                        }
                    }
                })
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
        let mut col = Column::new().spacing(5);

        let mut tree = self.tree.clone().unwrap_or(vec![]);
        tree.insert(0, FileItem {
            name: "..".to_owned(),
            path: self.current_path.parent().unwrap_or(&PathBuf::from("C:\\")).to_path_buf(),
            is_dir: true,
            size: None,
            last_modified: None,
            created: None,
            file_type: Some(FileType::Directory),
        });


        for item in tree {
            col = col.push(
                mouse_area(
                    container(
                        row![
                            container(
                                Image::new(image_from_type(item.clone().file_type.unwrap_or(FileType::File)))
                                    .width(14)
                                    .height(14)
                            )
                                .padding(Padding::default().top(3)),
                            text(format!("{}", item.name))
                                .width(Length::FillPortion(4))
                                .size(14),
                            text(if !item.is_dir { readable_size(item.size.unwrap_or(0)) } else { "-".to_owned() })
                                .width(Length::FillPortion(1))
                                .size(14),
                            text(readable_time(item.created))
                                .width(Length::FillPortion(2))
                                .size(14),
                            text(readable_time(item.last_modified))
                                .width(Length::FillPortion(2))
                                .size(14),
                        ]
                            .spacing(5)
                    )
                        .padding(5)
                        .style(if self.highlighted_file.as_ref() == Some(&item.name) {
                            |_: &_| iced::widget::container::Style {
                                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.06).into()),
                                ..Default::default()
                            }
                        } else {
                            |_: &_| iced::widget::container::Style {
                                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.02).into()),
                                ..Default::default()
                            }
                        })
                )
                    .on_press(Message::HighlightFile(item.clone()))
                    .on_double_click(Message::SelectFile(item))
                    
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