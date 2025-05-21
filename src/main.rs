mod platform;
mod views;
mod utils;
mod components;

use iced::font::Font;
use iced::widget::{row, text};
use iced::{Element, Subscription, Task, Theme};

use crate::views::explorer;
use crate::views::View;

pub fn main() -> iced::Result {
    iced::application(Filed::new, Filed::update, Filed::view)
        .subscription(Filed::subscription)
        .title(Filed::title)
        .theme(Filed::theme)
        .settings(iced::Settings {
            default_font: Font::with_name("Fantasy"),
            ..iced::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Loaded,
    Explorer(explorer::Message),
}

struct Filed {
    view: View,
}

impl Filed {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                view: View::Loading,
            },
            Task::perform(
                async {
                    // Simulate loading
                    // std::thread::sleep(std::time::Duration::from_secs(2));
                },
                |_| Message::Loaded,
            ),
        )
    }

    pub fn title(&self) -> String {
        match &self.view {
            View::Loading => "Filed - Loading...".to_string(),
            View::Explorer(explorer) => explorer.title(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded => {
                let (expl, task) = explorer::Explorer::open();
                self.view = View::Explorer(expl);   
                task.map(Message::Explorer)
            }
            Message::Explorer(msg) => {
                if let View::Explorer(expl) = &mut self.view {
                    expl.update(msg).map(Message::Explorer)
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.view {
            View::Loading => {
                row![
                    text("Loading..."),
                ]
                .into()
            }
            View::Explorer(explorer) => {
                explorer.view().map(Message::Explorer)
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.view {
            View::Loading => Subscription::none(),
            View::Explorer(explorer) => explorer.subscription().map(Message::Explorer),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dracula
    }
}