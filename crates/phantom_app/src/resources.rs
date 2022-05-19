mod input;
mod system;

pub use self::{input::Input, system::System};

use phantom_dependencies::{anyhow::Result, gilrs::Gilrs, winit::window::Window};
use phantom_gui::Gui;
use phantom_render::Renderer;

pub struct Resources<'a> {
    pub window: &'a mut Window,
    pub gilrs: &'a mut Gilrs,
    pub renderer: &'a mut Box<dyn Renderer>,
    pub gui: &'a mut Gui,
    pub input: &'a mut Input,
    pub system: &'a mut System,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<()> {
        Ok(self.window.set_cursor_grab(grab)?)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}
