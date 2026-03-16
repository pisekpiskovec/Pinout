use std::time::Duration;

use iced::theme::Mode;
use iced::widget::{button, column, container, row, rule, text, text_input, toggler};
use iced::Length::Fill;
use iced::{system, Element, Task, Theme};

use crate::config::Config;
use crate::pinstate::PinState;
use crate::server::PinServer;

const PORT_A_ADDR: u8 = 0x39;
const PORT_B_ADDR: u8 = 0x36;
const PORT_C_ADDR: u8 = 0x33;
const PORT_D_ADDR: u8 = 0x30;

#[derive(Debug)]
pub struct UInterface {
    bridge_address: String,
    pin_reset: bool,
    pin_a: u8,
    pin_b: u8,
    pin_c: u8,
    pin_d: u8,
    pin_state: PinState,
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
    SendReset(bool),
    SettingsBridgeChanged(String),
    ThemeChanged(Mode),
    TogglePin { port: u8, bit: u8 },
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
        let mut server = PinServer::new();
        server.start_server(&config.bridge_address).ok();

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
            server,
            pin_state: PinState::new(),
            pin_reset: false,
            pin_a: 0,
            pin_b: 0,
            pin_c: 0,
            pin_d: 0,
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
            bridge_address: self.bridge_address.clone(),
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
            Message::SendReset(pull) => {
                state.pin_reset = pull;
                state
                    .server
                    .send_data(0xFF, if pull { 0x01 } else { 0x00 })
                    .ok();
                state.status_message = Some("RESET sent".to_string());
            }
            Message::SettingsBridgeChanged(addr) => {
                state.temp_bridge_address = addr;
            }
            Message::RefreshData => {
                if let Some((addr, value)) = state.server.recive_data() {
                    state.pin_state.update_port(addr, value);
                    state.status_message =
                        Some(format!("Recieved: port {:#04X} = {:#04X}", addr, value));
                }
            }
            Message::TogglePin { port, bit } => {
                let new_value = match port {
                    PORT_A_ADDR => {
                        state.pin_a = Self::toggle_bit(state.pin_a, bit);
                        state.pin_a
                    }
                    PORT_B_ADDR => {
                        state.pin_b = Self::toggle_bit(state.pin_b, bit);
                        state.pin_b
                    }
                    PORT_C_ADDR => {
                        state.pin_c = Self::toggle_bit(state.pin_c, bit);
                        state.pin_c
                    }
                    PORT_D_ADDR => {
                        state.pin_d = Self::toggle_bit(state.pin_d, bit);
                        state.pin_d
                    }
                    _ => {
                        return Task::none();
                    }
                };
                state.server.send_data(port, new_value).ok();
            }
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

        let main_view = row![
            column![
                row![text("(XCK/T0) PB0".to_string()), toggler((self.pin_b >> 0) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 0 })],
                row![text("(T1) PB1".to_string()), toggler((self.pin_b >> 1) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 1 })],
                row![text("(INT2/AIN0) PB2".to_string()), toggler((self.pin_b >> 2) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 2 })],
                row![text("(OC0/AIN1) PB3".to_string()), toggler((self.pin_b >> 3) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 3 })],
                row![text("(|SS) PB4".to_string()), toggler((self.pin_b >> 4) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 4 })],
                row![text("(MOSI) PB5".to_string()), toggler((self.pin_b >> 5) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 5 })],
                row![text("(MISO) PB6".to_string()), toggler((self.pin_b >> 6) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 6 })],
                row![text("(SCK) PB7".to_string()), toggler((self.pin_b >> 7) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_B_ADDR, bit: 7 })],
                row![
                    text("|RESET".to_string()),
                    toggler(self.pin_reset).on_toggle(Message::SendReset),
                ],
                row![text("(RXD) PD0".to_string()), toggler((self.pin_d >> 0) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 0 })],
                row![text("(TXD) PD1".to_string()), toggler((self.pin_d >> 1) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 1 })],
                row![text("(INT0) PD2".to_string()), toggler((self.pin_d >> 2) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 2 })],
                row![text("(INT1) PD3".to_string()), toggler((self.pin_d >> 3) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 3 })],
                row![text("(OC1B) PD4".to_string()), toggler((self.pin_d >> 4) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 4 })],
                row![text("(OC1A) PD5".to_string()), toggler((self.pin_d >> 5) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 5 })],
                row![text("(ICP1) PD6".to_string()), toggler((self.pin_d >> 6) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 6 })],
            ]
            .spacing(8)
            .width(Fill),
            column![
                row![toggler((self.pin_a >> 0) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 0 }), text("PA0 (ADC0)".to_string()),],
                row![toggler((self.pin_a >> 1) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 1 }), text("PA1 (ADC1)".to_string()),],
                row![toggler((self.pin_a >> 2) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 2 }), text("PA2 (ADC2)".to_string()),],
                row![toggler((self.pin_a >> 3) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 3 }), text("PA3 (ADC3)".to_string()),],
                row![toggler((self.pin_a >> 4) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 4 }), text("PA4 (ADC4)".to_string()),],
                row![toggler((self.pin_a >> 5) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 5 }), text("PA5 (ADC5)".to_string()),],
                row![toggler((self.pin_a >> 6) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 6 }), text("PA6 (ADC6)".to_string()),],
                row![toggler((self.pin_a >> 7) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_A_ADDR, bit: 7 }), text("PA7 (ADC7)".to_string()),],
                row![toggler((self.pin_c >> 7) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 7 }), text("PC7 (TOSC2)".to_string())],
                row![toggler((self.pin_c >> 6) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 6 }), text("PC6 (TOSC1)".to_string())],
                row![toggler((self.pin_c >> 5) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 5 }), text("PC5 (TDI)".to_string())],
                row![toggler((self.pin_c >> 4) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 4 }), text("PC4 (TDO)".to_string())],
                row![toggler((self.pin_c >> 3) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 3 }), text("PC3 (TMS)".to_string())],
                row![toggler((self.pin_c >> 2) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 2 }), text("PC2 (TCK)".to_string())],
                row![toggler((self.pin_c >> 1) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 1 }), text("PC1 (SDA)".to_string())],
                row![toggler((self.pin_c >> 0) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_C_ADDR, bit: 0 }), text("PC0 (SCL)".to_string())],
                row![toggler((self.pin_d >> 7) & 1 == 1).on_toggle(|_| Message::TogglePin { port: PORT_D_ADDR, bit: 7 }), text("PD7 (OC2)".to_string())],
            ]
            .spacing(8)
            .width(Fill)
        ]
        .height(Fill);

        content = content.push(main_view);
        content = content.push(rule::horizontal(2));

        let mut status_bar = row![];
        if let Some(status) = self.status_message.as_ref() {
            status_bar = status_bar.push(text(status).width(Fill));
        } else {
            status_bar = status_bar.push(text("").width(Fill));
        }
        status_bar = status_bar.push(text!(
            "Network status: {}",
            match self.server.is_connected() {
                true => "Connected",
                false => "Disconnected",
            }
        ));
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
                text_input("", &self.temp_bridge_address).on_input(Message::SettingsBridgeChanged)
            ]
            .spacing(4)
            .padding(4),
        );
        container(content).into()
    }

    fn toggle_bit(value: u8, bit: u8) -> u8 {
        value ^ (1 << bit)
    }
}
