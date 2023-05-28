pub mod directory_event;
pub mod relay_descriptor;
pub mod routers;
pub mod user_descriptor;

pub use directory_event::*;
pub use relay_descriptor::*;
pub use routers::*;
pub use user_descriptor::*;

pub const DIRECTORY_ADDRESS: &'static str = "127.0.0.1:8100";

pub fn start_directory() {

}