#![windows_subsystem = "windows"]

mod app;
mod db;
mod db_migrations;
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

use iced::{event, mouse, Element, Event, Size, Subscription, Task, Theme};
use crate::app::{AppState, Message, WorkspaceMessage, APP_NAME, APP_ACRONYM};
use crate::project_manager::ProjectManager;
use crate::state::{ToastKind, DbAction, DemoResetScope};

pub fn main() -> iced::Result {
    let _ = crate::logger::init();
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size(Size::new(1600.0, 950.0))
        .run()
}

struct App {
    state: AppState,
    db: Option<db::Database>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        crate::logger::info("App starting in Launcher Mode...");
        let mut app = Self {
            state: AppState::default(),
            db: None,
        };
        app.state.route = crate::app::Route::Workspaces;
        let load_projs =
            Task::perform(async { ProjectManager::load_projects() }, Message::ProjectsLoaded);
        (app, load_projs)
    }

    fn title(&self) -> String {
        if let Some(p) = &self.state.active_project {
            format!("{} - {} ({})", p.name, APP_NAME, APP_ACRONYM)
        } else {
            format!("{} ({})", APP_NAME, APP_ACRONYM)
        }
    }

    fn theme(&self) -> Theme {
        app::app_theme(&self.state)
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = Vec::new();
        if let app::PmState::Dragging { .. } = self.state.pm_state {
            subs.push(event::listen_with(|event, _status, _window| match event {
                Event::Mouse(mouse::Event::CursorMoved { position }) => Some(
                    Message::GlobalEvent(Event::Mouse(mouse::Event::CursorMoved { position })),
                ),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Some(
                    Message::GlobalEvent(Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    ))),
                ),
                _ => None,
            }));
        }
        if !self.state.toasts.is_empty() {
            subs.push(iced::time::every(std::time::Duration::from_secs(1)).map(Message::Tick));
        }
        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Message> {
        app::view(&self.state)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = Vec::new();
        match message {
            Message::ProjectsLoaded(projs) => {
                self.state.projects = projs;
            }

            Message::DbLoaded(result) => match result {
                Ok(db) => {
                    crate::logger::info("Database connected.");
                    self.db = Some(db);
                    tasks.push(self.fetch_universes());
                    tasks.push(self.fetch_boards());
                }
                Err(e) => {
                    crate::logger::error(&format!("Failed to connect to DB: {}", e));
                    self.state
                        .show_toast("Failed to open database", ToastKind::Error);
                }
            },

            Message::ProjectCreated(result) => match result {
                Ok(_) => {
                    self.state.is_creating_project = false;
                    tasks.push(Task::perform(
                        async { ProjectManager::load_projects() },
                        Message::ProjectsLoaded,
                    ));
                    self.state.show_toast("Workspace created", ToastKind::Success);
                }
                Err(e) => {
                    crate::logger::error(&format!("Project create failed: {}", e));
                    self.state.show_toast(e, ToastKind::Error);
                }
            },

            Message::Workspace(WorkspaceMessage::Open(id)) => {
                if let Some(proj) = self.state.projects.iter().find(|p| p.id == id).cloned() {
                    self.state.active_project = Some(proj.clone());
                    self.state.route = crate::app::Route::Overview;

                    let pid = proj.id.clone();
                    std::thread::spawn(move || ProjectManager::update_last_opened(&pid));

                    let path = std::path::PathBuf::from(proj.path.clone());
                    tasks.push(Task::perform(
                        async move { db::Database::connect(path).await.map_err(|e| e.to_string()) },
                        Message::DbLoaded,
                    ));
                }
            }

            Message::Workspace(WorkspaceMessage::CreateConfirm) => {
                let name = self.state.new_project_name.clone();
                tasks.push(Task::perform(
                    async move { ProjectManager::create_project(name) },
                    Message::ProjectCreated,
                ));
            }

            Message::Workspace(WorkspaceMessage::Delete(id)) => {
                let pid = id.clone();
                tasks.push(Task::perform(
                    async move {
                        let _ = ProjectManager::delete_project(&pid);
                        ProjectManager::load_projects()
                    },
                    Message::ProjectsLoaded,
                ));
            }

            Message::Workspace(WorkspaceMessage::CloseProject) => {
                self.state.active_project = None;
                self.db = None;
                self.state = crate::state::AppState::default();
                tasks.push(Task::perform(
                    async { ProjectManager::load_projects() },
                    Message::ProjectsLoaded,
                ));
            }

            _ => {
                // --- Action Done handling (adds contextual toasts) ---
                if let Message::ActionDone(result) = &message {
                    let inflight = self.state.db_inflight.clone();
                    self.state.db_inflight = None;

                    match result {
                        Ok(_) => {
                            // Force reload
                            self.state.data_dirty = true;
                            self.state.loaded_creatures_universe = None;
                            self.state.loaded_locations_universe = None;
                            self.state.loaded_timeline_universe = None;
                            self.state.pm_data = None;

                            if let Some(action) = inflight {
                                match action {
                                    DbAction::ResetDemoDataScoped(_, scope) => {
                                        let msg = match scope {
                                            DemoResetScope::All => "Demo reset complete: Bestiary(7), Locations(7), Timeline(5 eras/15 events), PM Tools(6 cards)",
                                            DemoResetScope::Timeline => "Timeline reset complete: 5 eras / 15 events",
                                            DemoResetScope::Locations => "Locations reset complete: 7 locations",
                                            DemoResetScope::Bestiary => "Bestiary reset complete: 7 creatures",
                                            DemoResetScope::PmTools => "PM Tools reset complete: 6 cards",
                                        };
                                        self.state.show_toast(msg, ToastKind::Success);
                                    }
                                    DbAction::InjectDemoData(_) => {
                                        self.state.show_toast("Demo data injected", ToastKind::Success);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            self.state
                                .show_toast(format!("Action failed: {}", e), ToastKind::Error);
                        }
                    }
                }

                controllers::root_controller::update(&mut self.state, message.clone());

                if self.state.active_project.is_some() {
                    match &message {
                        Message::GlobalEvent(e) => match e {
                            Event::Mouse(mouse::Event::CursorMoved { position }) => controllers::root_controller::update(
                                &mut self.state,
                                app::Message::MouseMoved(*position),
                            ),
                            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => controllers::root_controller::update(
                                &mut self.state,
                                app::Message::MouseReleased,
                            ),
                            _ => {}
                        },
                        _ => {}
                    }
                    tasks.extend(controllers::root_controller::post_event_tasks(
                        &mut self.state,
                        &self.db,
                    ));
                }
            }
        }
        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    fn fetch_universes(&self) -> Task<Message> {
        let Some(db) = self.db.clone() else { return Task::none(); };
        Task::perform(
            async move { db.get_all_universes().await.map_err(|e| e.to_string()) },
            Message::UniversesFetched,
        )
    }

    fn fetch_boards(&self) -> Task<Message> {
        let Some(db) = self.db.clone() else { return Task::none(); };
        Task::perform(
            async move { db.get_all_boards().await.map_err(|e| e.to_string()) },
            Message::BoardsFetched,
        )
    }
}
