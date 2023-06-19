use crate::decoder::*;
use crate::encoder::*;
use iced::{
    alignment::*,
    theme::Theme,
    widget::{column, row, Button, Column, Container, Row, Space, Text, Toggler},
    Element, Length, Renderer, Sandbox,
};
use rfd::FileDialog;
use std::fmt::{Debug, Formatter, Result};

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    ThemeChanged(bool),
    InterceptFileChanged(bool),
    DecodeButtonPressed,
    EncodeButtonPressed,
    DecodeOutputButtonPressed,
    EncodeOutputButtonPressed,
}

#[derive(Default, Clone, PartialEq)]
pub enum AppTheme {
    Light,
    #[default]
    Dark,
}

impl Debug for AppTheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            AppTheme::Dark => {
                write!(f, "Dark")
            }
            AppTheme::Light => {
                write!(f, "Light")
            }
        }
    }
}

#[derive(Default)]
pub enum AppState {
    #[default]
    Home,
    DecodeHome,
    EncodeHome,
}

#[derive(Default)]
pub struct App {
    pub theme: AppTheme,
    state: AppState,
    intercept_file: bool, // TODO Use this value
    decoder: Decoder,
    encoder: Encoder,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App::default()
    }

    fn title(&self) -> String {
        String::from("Geometry Dash Save Explorer")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Reset => {
                self.state = AppState::Home;
                self.intercept_file = false;
                self.decoder = Decoder::new_no_file_selected();
                self.encoder = Encoder::new_no_file_selected();
            }
            Message::ThemeChanged(value) => {
                self.theme = if value {
                    AppTheme::Dark
                } else {
                    AppTheme::Light
                }
            }
            Message::InterceptFileChanged(value) => {
                self.intercept_file = value;
            }
            Message::DecodeButtonPressed => {
                if let Some(file_path) = FileDialog::new()
                    .add_filter("Geometry Dash Save", &["dat"])
                    .set_directory("~")
                    .set_title("Select The Geometry Dash Save")
                    .pick_file()
                {
                    self.decoder = Decoder::new_file_selected(file_path);
                    self.state = AppState::DecodeHome;
                }
            }
            Message::DecodeOutputButtonPressed => match self.decoder.output() {
                Ok(result) => match result {
                    DecoderOutputResult::FileCreated => self.state = AppState::Home,
                    DecoderOutputResult::FileNotCreated => {}
                },
                Err(e) => {
                    println!("{}", e);
                }
            },
            Message::EncodeButtonPressed => {
                if let Some(file_path) = FileDialog::new()
                    .add_filter("Geometry Dash Raw Save", &["xml"])
                    .set_directory("~")
                    .set_title("Select The Geometry Dash Raw Save")
                    .pick_file()
                {
                    self.encoder = Encoder::new_file_selected(file_path);
                    self.state = AppState::EncodeHome;
                }
            }
            Message::EncodeOutputButtonPressed => match self.encoder.output() {
                Ok(result) => match result {
                    EncoderOutputResult::FileCreated => self.state = AppState::Home,
                    EncoderOutputResult::FileNotCreated => {}
                },
                Err(e) => {
                    println!("{}", e);
                }
            },
        }
    }

    fn theme(&self) -> Theme {
        match self.theme {
            AppTheme::Dark => Theme::Dark,
            AppTheme::Light => Theme::Light,
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let theme_toggle: Toggler<'_, Message, Renderer> = Toggler::new(
            format!("{:?}", self.theme),
            self.theme == AppTheme::Dark,
            Message::ThemeChanged,
        )
        .width(Length::Shrink)
        .text_alignment(Horizontal::Right)
        .spacing(8)
        .text_size(16);

        let menu_left: Column<'_, Message, Renderer> = column![Button::new("Home")
            .height(Length::Fixed(32.0))
            .on_press(Message::Reset)]
        .width(Length::Fill)
        .align_items(Alignment::Start);

        let menu_center: Column<'_, Message, Renderer> =
            column![Text::new("Geometry Dash Save Explorer")
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center)
                .size(32)]
            .width(Length::Shrink)
            .align_items(Alignment::Center);

        let menu_right: Column<'_, Message, Renderer> = column![
            Space::new(Length::Fill, Length::FillPortion(3)),
            theme_toggle,
            Space::new(Length::Fill, Length::FillPortion(3))
        ]
        .width(Length::Fill)
        .align_items(Alignment::End);

        let menubar: Row<'_, Message, Renderer> = row![menu_left, menu_center, menu_right]
            .spacing(8)
            .padding(8)
            .height(48);

        let body: Container<'_, Message, Renderer> = match self.state {
            AppState::Home => {
                let decode_button: Button<'_, Message, Renderer> =
                    Button::new("Decode Save File").on_press(Message::DecodeButtonPressed);
                let encode_button: Button<'_, Message, Renderer> =
                    Button::new("Encode Save File").on_press(Message::EncodeButtonPressed);
                let intercept_toggle: Toggler<'_, Message, Renderer> = Toggler::new(
                    String::from("Intercept"),
                    self.intercept_file,
                    Message::InterceptFileChanged,
                )
                .width(Length::Shrink)
                .text_alignment(Horizontal::Right)
                .spacing(8)
                .text_size(16);

                Container::new(
                    column![
                        decode_button,
                        Space::new(Length::Shrink, Length::Fixed(16.0)),
                        encode_button,
                        Space::new(Length::Shrink, Length::Fixed(16.0)),
                        intercept_toggle
                    ]
                    .align_items(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
            }
            AppState::DecodeHome => {
                let output_button: Button<'_, Message, Renderer> =
                    Button::new("Save").on_press(Message::DecodeOutputButtonPressed);

                Container::new(column![output_button])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
            }
            AppState::EncodeHome => {
                let output_button: Button<'_, Message, Renderer> =
                    Button::new("Save").on_press(Message::EncodeOutputButtonPressed);

                Container::new(column![output_button])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
            }
        };

        let content: Column<'_, Message, Renderer> = column![menubar, body].spacing(16).padding(16);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
