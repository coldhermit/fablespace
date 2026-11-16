//! Shared type, spacing, and panels for the maker window.

use iced::widget::button::{Status, Style as ButtonStyle};
use iced::widget::{button, column, container, text};
use iced::{Background, Border, Color, Element, Length, Shadow};

use super::Message;

pub const INK: Color = Color::from_rgb(0.94, 0.91, 0.84);
pub const MUTED: Color = Color::from_rgb(0.62, 0.7, 0.62);
pub const GOLD: Color = Color::from_rgb(0.9, 0.74, 0.36);
pub const PANEL: Color = Color::from_rgb(0.11, 0.16, 0.13);
pub const LINE: Color = Color::from_rgb(0.28, 0.4, 0.32);

pub fn card<'a>(title: &str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(column![text(title.to_string()).size(12).color(MUTED), body].spacing(10))
        .padding(14)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(PANEL)),
            text_color: Some(INK),
            border: Border {
                color: LINE,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

pub fn bar<'a>(body: Element<'a, Message>) -> Element<'a, Message> {
    container(body)
        .padding([10, 14])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.12, 0.1))),
            text_color: Some(INK),
            border: Border {
                color: LINE,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

pub fn nav_item(label: String, on: bool, message: Message) -> Element<'static, Message> {
    let color = if on { GOLD } else { INK };
    button(text(label).size(15).color(color))
        .on_press(message)
        .width(Length::Fill)
        .padding([8, 12])
        .style(move |_theme, status| {
            let hot = matches!(status, Status::Hovered | Status::Pressed);
            ButtonStyle {
                background: Some(Background::Color(if on {
                    Color::from_rgb(0.2, 0.28, 0.2)
                } else if hot {
                    Color::from_rgb(0.16, 0.22, 0.18)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.0)
                })),
                text_color: color,
                border: Border {
                    radius: 6.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
            }
        })
        .into()
}

pub fn action(label: &str, message: Option<Message>) -> Element<'static, Message> {
    button(text(label.to_string()).size(14))
        .padding([8, 14])
        .on_press_maybe(message)
        .style(|_theme, status| {
            let hot = matches!(status, Status::Hovered | Status::Pressed);
            let disabled = matches!(status, Status::Disabled);
            ButtonStyle {
                background: Some(Background::Color(if disabled {
                    Color::from_rgb(0.16, 0.18, 0.16)
                } else if hot {
                    Color::from_rgb(0.42, 0.32, 0.14)
                } else {
                    Color::from_rgb(0.28, 0.22, 0.12)
                })),
                text_color: if disabled { MUTED } else { INK },
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
            }
        })
        .into()
}

pub fn group_label(label: &str) -> Element<'static, Message> {
    text(label.to_string()).size(11).color(MUTED).into()
}
