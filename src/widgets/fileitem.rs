use std::{fs::DirEntry, path::PathBuf, time::SystemTime};

use iced::{widget::{container, mouse_area, row, text, Image}, Color, Element, Length, Padding, Task};

use crate::utils::{file_type_from_extension, image_from_type, readable_size, readable_time};
use crate::views::explorer::Message;

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
pub struct FileData {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub last_modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub file_type: Option<FileType>,
}

impl FileData {
    pub fn new(entry: DirEntry) -> Self {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let is_dir = entry.file_type().unwrap().is_dir();
        let metadata = entry.metadata().unwrap();
        let created = metadata.created().ok();
        let last_modified = metadata.modified().ok();
        let size = if is_dir { None } else { Some(metadata.len()) };
        Self {
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
            },
        }
    }

    pub fn parent(current_path: PathBuf) -> Self {
        Self {
            name: "..".to_owned(),
            path: current_path.parent().unwrap_or(&PathBuf::from("C:\\")).to_path_buf(),
            is_dir: true,
            size: None,
            last_modified: None,
            created: None,
            file_type: Some(FileType::Directory),
        }
    }
}

pub struct FileItem<Message> {
    pub data: FileData,

    highlighted: bool,
    on_select: Option<Box<dyn Fn(FileData) -> Message>>,
    on_open: Option<Box<dyn Fn(FileData) -> Message>>,
}

impl FileItem<Message> {
    pub fn new(entry: DirEntry) -> Self {
        Self {
            data: FileData::new(entry),
            highlighted: false,
            on_select: None,
            on_open: None,
        }
    }

    pub fn parent(current_path: PathBuf) -> Self {
        Self {
            data: FileData::parent(current_path),
            highlighted: false,
            on_select: None,
            on_open: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            _ => Task::none(),
        }
    }

    pub fn view<'a>(self) -> Element<'a, Message> {
        let mut row = mouse_area(
            container(
                row![
                    container(
                        Image::new(image_from_type(self.data.clone().file_type.unwrap_or(FileType::Unknown)))
                            .width(14)
                            .height(14)
                    )
                        .padding(Padding::default().top(3)),
                    text(format!("{}", self.data.name))
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                        .width(Length::FillPortion(4))
                        .size(14),
                    text(if !self.data.is_dir { readable_size(self.data.size.unwrap_or(0)) } else { "-".to_owned() })
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                        .width(Length::FillPortion(1))
                        .size(14),
                    text(readable_time(self.data.created))
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                        .width(Length::FillPortion(2))
                        .size(14),
                    text(readable_time(self.data.last_modified))
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8))
                        .width(Length::FillPortion(2))
                        .size(14),
                ]
                    .spacing(5)
            )
                .padding(5)
                .style(if self.highlighted {
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
        );

        if let Some(msg) = self.on_select {
            row = row.on_press(msg(self.data.clone()));
        }
        if let Some(msg) = self.on_open.as_ref() {
            row = row.on_double_click(msg(self.data.clone()));
        }

        row.into()
    }

    pub fn is_highlighted(mut self, highlight: bool) -> Self {
        self.highlighted = highlight;
        self
    }

    pub fn on_select(mut self, msg: Box<dyn Fn(FileData) -> Message>) -> Self {
        self.on_select = Some(msg);
        self
    }

    pub fn on_open(mut self, msg: Box<dyn Fn(FileData) -> Message>) -> Self {
        self.on_open = Some(msg);
        self
    }
}

impl<'a> From<FileItem<Message>>
    for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(item: FileItem<Message>) -> Self {
        item.view()
    }
}

impl<'a> From<FileData>
    for FileItem<Message>
where
    Message: 'a + Clone,
{
    fn from(item: FileData) -> Self {
        FileItem {
            data: item,
            highlighted: false,
            on_select: None,
            on_open: None,
        }
    }
}