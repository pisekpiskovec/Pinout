use std::time::Duration;

use iced::theme::Mode;
use iced::widget::{button, column, container, row, rule, text, text_input};
use iced::Length::Fill;
use iced::{system, Element, Task, Theme};

use crate::config::Config;
use crate::server::PinServer;

#[derive(Debug)]
pub struct UInterface {
    bridge_address: String,
    server: PinServer,
    show_settings: bool,
    status_message: Option<String>,
    temp_bridge_address: String,
    theme: Theme,
    theme_mode: Mode,
}

#[derive(Debug, Clone)]
pub enum Message {
    CloseSettings,
    OpenSettings,
    RefreshData,
    SaveSettings,
    SettingsBridgeChanged(String),
    ThemeChanged(Mode),
}

impl UInterface {
    fn mode_to_theme(mode: Mode) -> Theme {
        match mode {
            Mode::None => Theme::Ferra,
            Mode::Light => Theme::GruvboxLight,
            Mode::Dark => Theme::SolarizedDark,
        }
    }

    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();

        Self {
            theme_mode: match config.theme.mode.as_str() {
                "Light" => Mode::Light,
                "Dark" => Mode::Dark,
                _ => Mode::None,
            },
            theme: Theme::Dark,
            show_settings: false,
            status_message: None,
            bridge_address: config.bridge_address,
            temp_bridge_address: Config::load().unwrap_or_default().bridge_address,
            server: PinServer::new(),
        }
    }

    fn save_config(&self) -> Result<(), String> {
        let config = Config {
            theme: crate::config::ThemeConfig {
                mode: match self.theme_mode {
                    Mode::Light => "Light".to_string(),
                    Mode::Dark => "Dark".to_string(),
                    Mode::None => String::new(),
                },
            },
            bridge_address: self.bridge_address.repeat(0),
        };
        config.save()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let theme_sub = system::theme_changes().map(Message::ThemeChanged);
        let data_sub = iced::time::every(Duration::from_secs(1)).map(|_| Message::RefreshData);
        iced::Subscription::batch(vec![theme_sub, data_sub])
    }

    pub fn theme(&self) -> Theme {
        match self.theme_mode {
            Mode::None => Theme::Ferra,
            Mode::Light => Theme::GruvboxLight,
            Mode::Dark => Theme::SolarizedDark,
        }
    }

    pub fn update(state: &mut UInterface, message: Message) -> Task<Message> {
        match message {
            Message::ThemeChanged(mode) => {
                state.theme = UInterface::mode_to_theme(mode);
                state.theme_mode = mode;
            }
            Message::OpenSettings => {
                state.show_settings = true;
            }
            Message::CloseSettings => {
                state.show_settings = false;
            }
            Message::SaveSettings => {
                state.bridge_address = state.temp_bridge_address.trim().to_string();
                let _ = state.save_config();
                state.show_settings = false;
            }
            Message::SettingsBridgeChanged(addr) => {
                state.temp_bridge_address = addr;
            }
            Message::RefreshData => {
                state.server.recive_data();
            }
        }
        match state.server.is_connected() {
            true => state.status_message = Some("Connected".to_string()),
            false => state.status_message = Some("Disconnected".to_string()),
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.show_settings {
            self.view_settings()
        } else {
            self.view_main()
        }
    }

    fn view_main(&self) -> Element<'_, Message> {
        let mut content = column![].spacing(2).padding(4);

        let header = row![
            text("Pinout").size(36).width(Fill),
            button(text("Config")).on_press(Message::OpenSettings)
        ]
        .spacing(8);
        content = content.push(header);

        content = content.push(rule::horizontal(2));

        let toolbar = row![].spacing(8).padding(4);
        content = content.push(toolbar);
        content = content.push(rule::horizontal(2));

        let main_view = row![].height(Fill);

        content = content.push(main_view);
        content = content.push(rule::horizontal(2));

        let mut status_bar = row![];
        if let Some(status) = self.status_message.as_ref() {
            status_bar = status_bar.push(text(status).width(Fill));
        } else {
            status_bar = status_bar.push(text("").width(Fill));
        }
        content = content.push(status_bar);

        container(content).into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let mut content = column![].spacing(2).padding(4);

        let header = row![
            text("Pinout").size(36),
            text(env!("CARGO_PKG_VERSION")).width(Fill),
            button(text("Cancel")).on_press(Message::CloseSettings),
            button(text("Save")).on_press(Message::SaveSettings),
        ]
        .spacing(8);
        content = content.push(header);

        content = content.push(rule::horizontal(2));

        content = content.push(
            row![
                text("Hardware bridge address:"),
                text_input("", &self.bridge_address).on_input(Message::SettingsBridgeChanged)
            ]
            .spacing(4)
            .padding(4),
        );
        container(content).into()
    }
}
