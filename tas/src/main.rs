#![windows_subsystem = "windows"]

mod app;
mod db;
mod db_seed;
mod logger;
mod model;
mod pages;
mod ui;
mod controllers;
mod project_manager;
mod messages;
mod state;
mod editors;

pub fn main() -> iced::Result {
    controllers::ui_controller::run()
}
