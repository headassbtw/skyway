use timeline::FrontendTimelineView;

pub mod landing;
pub mod timeline;
pub mod thread;

pub enum FrontendMainView {
	Login(),
	Timeline(FrontendTimelineView),
}