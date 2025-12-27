#![allow(unused_imports)]

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
pub mod the_forge; // <--- NUEVO MODULO

// --- RE-EXPORTS ---
pub use overview::overview;
pub use universe_list::universe_list;
pub use universe_detail::universe_detail;
pub use bestiary::bestiary;
pub use launcher::launcher_view;
// Reemplazamos el stub con la vista real
pub use the_forge::the_forge_view;

pub use stubs::assets_stub;
pub use stubs::account_stub;

pub type E<'a> = iced::Element<'a, crate::messages::Message>;
