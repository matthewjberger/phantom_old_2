use phantom_dependencies::{anyhow::Result, gilrs::Gilrs, winit::window::Window};

pub struct Resources<'a> {
    pub window: &'a mut Window,
    pub gilrs: &'a mut Gilrs,
}

impl<'a> Resources<'a> {
    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<()> {
        Ok(self.window.set_cursor_grab(grab)?)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible)
    }
}
