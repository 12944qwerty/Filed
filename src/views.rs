pub mod explorer;

pub use explorer::{Explorer, FileItem};

pub enum View {
    Loading,
    Explorer(Explorer),
}