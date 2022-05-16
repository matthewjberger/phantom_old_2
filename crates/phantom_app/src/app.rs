use phantom_dependencies::{
    anyhow::Result,
    env_logger,
    image::io::Reader,
    log,
    winit::{
        dpi::PhysicalSize,
        event::*,
        event_loop::{ControlFlow, EventLoop},
        window::{Icon, WindowBuilder},
    },
};

use crate::{Resources, State, StateMachine};

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub is_fullscreen: bool,
    pub title: String,
    pub icon: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            is_fullscreen: false,
            title: "Phantom Editor".to_string(),
            icon: None,
        }
    }
}

pub fn run(initial_state: impl State + 'static, config: AppConfig) -> Result<()> {
    env_logger::init();

    log::info!("Phantom app started");

    let event_loop = EventLoop::new();
    let mut window_builder = WindowBuilder::new()
        .with_title(config.title.to_string())
        .with_inner_size(PhysicalSize::new(config.width, config.height));

    if let Some(icon_path) = config.icon.as_ref() {
        let image = Reader::open(icon_path)?.decode()?.into_rgba8();
        let (width, height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), width, height)?;
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let mut window = window_builder.build(&event_loop)?;

    let mut state_machine = StateMachine::new(initial_state);

    event_loop.run(move |event, _, control_flow| {
        let mut resources = Resources {
            window: &mut window,
        };
        if let Err(error) = run_loop(&mut state_machine, &event, &mut resources, control_flow) {
            log::error!("Application error: {}", error);
        }
    });
}

fn run_loop(
    state_machine: &mut StateMachine,
    event: &Event<()>,
    resources: &mut Resources,
    control_flow: &mut ControlFlow,
) -> Result<()> {
    if !state_machine.is_running() {
        state_machine.start(resources)?;
    }

    state_machine
        .handle_event(resources, &event)
        .expect("Failed to handle event!");

    match event {
        Event::MainEventsCleared => {
            state_machine.update(resources)?;
        }

        Event::WindowEvent {
            ref event,
            window_id,
        } if *window_id == resources.window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => {
                if let (Some(VirtualKeyCode::Escape), ElementState::Pressed) =
                    (input.virtual_keycode, input.state)
                {
                    *control_flow = ControlFlow::Exit;
                }
                state_machine.current_state()?.on_key(resources, *input)?;
            }

            WindowEvent::MouseInput { button, state, .. } => {
                state_machine
                    .current_state()?
                    .on_mouse(resources, button, state)?;
            }

            WindowEvent::DroppedFile(ref path) => {
                state_machine
                    .current_state()?
                    .on_file_dropped(resources, path)?;
            }

            _ => {}
        },
        _ => {}
    }
    Ok(())
}
