pub mod overview;
pub mod workspaces;
pub mod universe_list;
pub mod universe_detail;
pub mod bestiary;
pub mod locations;
pub mod timeline;
pub mod pm_list;
pub mod pm_board;
pub mod launcher;
pub mod stubs;

// --- RE-EXPORTS ---
// Esto permite llamar a `pages::overview(...)` en lugar de `pages::overview::overview(...)`
pub use overview::overview;
pub use universe_list::universe_list;
pub use universe_detail::universe_detail;
pub use bestiary::bestiary;
pub use launcher::launcher_view;

// Stubs (Forge, Assets, Account)
pub use stubs::forge_stub;
pub use stubs::assets_stub;
pub use stubs::account_stub;
pub use stubs::workspaces_stub; // En caso de que se use el stub
pub use stubs::timeline_stub;   // En caso de que se use el stub

// Element alias
pub type E<'a> = iced::Element<'a, crate::messages::Message>;