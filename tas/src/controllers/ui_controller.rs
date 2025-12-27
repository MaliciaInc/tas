use iced::{event, mouse, Element, Event, Size, Subscription, Task, Theme};

use crate::app::{AppState, Message, WorkspaceMessage, APP_ACRONYM, APP_NAME};
use crate::project_manager::ProjectManager;
use crate::state::ToastKind;

use crate::db::Database;

pub fn run() -> iced::Result {
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
    db: Option<Database>,
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
        crate::app::app_theme(&self.state)
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = Vec::new();

        // Drag/Drop mouse tracking (PM)
        if let crate::app::PmState::Dragging { .. } = self.state.pm_state {
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

        // Toast TTL tick
        if !self.state.toasts.is_empty() {
            subs.push(iced::time::every(std::time::Duration::from_secs(1)).map(Message::Tick));
        }

        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Message> {
        crate::app::view(&self.state)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = Vec::new();

        match message {
            // Projects list loaded
            Message::ProjectsLoaded(projs) => {
                self.state.projects = projs;
            }

            // DB connected
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

            // Project creation result
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

            // Workspace commands that require side effects (Tasks)
            Message::Workspace(WorkspaceMessage::Open(id)) => {
                if let Some(proj) = self.state.projects.iter().find(|p| p.id == id).cloned() {
                    self.state.active_project = Some(proj.clone());
                    self.state.route = crate::app::Route::Overview;

                    let pid = proj.id.clone();
                    std::thread::spawn(move || ProjectManager::update_last_opened(&pid));

                    let path = std::path::PathBuf::from(proj.path.clone());
                    tasks.push(Task::perform(
                        async move { Database::connect(path).await.map_err(|e| e.to_string()) },
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

            // Everything else: goes through modular controllers
            _ => {
                // ActionDone side effects moved out to a dedicated controller
                if let Message::ActionDone(result) = &message {
                    crate::controllers::action_done_controller::handle_action_done(
                        &mut self.state,
                        result,
                    );
                }

                // Message interpretation (state mutation + delegation)
                crate::controllers::messages_controller::update(&mut self.state, message.clone());

                // Global mouse events to PM drag logic via messages_controller
                if self.state.active_project.is_some() {
                    if let Message::GlobalEvent(e) = &message {
                        match e {
                            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                                crate::controllers::messages_controller::update(
                                    &mut self.state,
                                    crate::app::Message::MouseMoved(*position),
                                );
                            }
                            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                                crate::controllers::messages_controller::update(
                                    &mut self.state,
                                    crate::app::Message::MouseReleased,
                                );
                            }
                            _ => {}
                        }
                    }

                    // Post-event scheduler (db queue + lazy fetch)
                    tasks.extend(crate::controllers::post_event_tasks_controller::post_event_tasks(
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
