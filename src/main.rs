mod config;
mod ui;

use crate::ui::UInterface;

fn main() -> iced::Result {
    iced::application(UInterface::new, UInterface::update, UInterface::view)
        .title("Pinout")
        .theme(UInterface::theme)
        .subscription(UInterface::subscription)
        .run()
}
