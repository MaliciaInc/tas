pub mod overview;
pub mod universe_list;
pub mod universe_detail;
pub mod bestiary;
pub mod stubs;

use iced::Element;
use crate::app::Message;

pub type E<'a> = Element<'a, Message>;

pub use overview::overview;
pub use universe_list::universe_list;
pub use universe_detail::universe_detail;
pub use bestiary::bestiary;

pub use stubs::{
    workspaces_stub, timeline_stub, forge_stub, pm_stub, assets_stub, account_stub,
};
