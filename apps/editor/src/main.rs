use phantom::{
    app::{run, AppConfig, Resources, State, Transition},
    dependencies::{
        anyhow::Result,
        log,
        winit::event::{ElementState, Event, KeyboardInput, MouseButton},
    },
};

#[derive(Default)]
struct Editor;

impl State for Editor {
    fn on_start(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Starting the Phantom editor");
        Ok(())
    }

    fn on_stop(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Stopping the Phantom editor");
        Ok(())
    }

    fn on_pause(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Editor paused");
        Ok(())
    }

    fn on_resume(&mut self, _resources: &mut Resources) -> Result<()> {
        log::info!("Editor unpaused");
        Ok(())
    }

    fn update(&mut self, _resources: &mut Resources) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        path: &std::path::PathBuf,
    ) -> Result<()> {
        log::info!(
            "File dropped: {}",
            path.as_os_str().to_str().expect("Failed to convert path!")
        );
        Ok(())
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        button: &MouseButton,
        button_state: &ElementState,
    ) -> Result<()> {
        log::info!("Mouse event: {:#?} {:#?}", button, button_state,);
        Ok(())
    }

    fn on_key(&mut self, _resources: &mut Resources, input: KeyboardInput) -> Result<()> {
        log::info!("Key event received: {:#?}", input);
        Ok(())
    }

    fn handle_event(
        &mut self,
        _resources: &mut Resources,
        _event: &Event<()>,
    ) -> Result<Transition> {
        // log::info!("Event received: {:#?}", event);
        Ok(Transition::None)
    }
}

fn main() -> Result<()> {
    run(Editor::default(), AppConfig::default())
}
