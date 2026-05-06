mod app;
mod assets;
mod components;
mod crypto;
mod db;
mod export;
mod favicon;
#[allow(dead_code)]
mod importers;
mod models;
mod screens;

pub(crate) use app::{Message, Screen};

pub fn main() -> iced::Result {
    app::run()
}
