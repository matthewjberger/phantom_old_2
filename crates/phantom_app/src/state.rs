use crate::Resources;
use phantom_dependencies::{
    anyhow::{Context, Result},
    gilrs::Event as GilrsEvent,
    winit::event::{ElementState, Event, KeyboardInput, MouseButton},
};
use std::path::PathBuf;

pub struct EmptyState {}
impl State for EmptyState {}

pub trait State {
    fn on_start(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn on_stop(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn on_pause(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn on_resume(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _resources: &mut Resources) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_file_dropped(
        &mut self,
        _resources: &mut Resources,
        _path: &PathBuf,
    ) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_mouse(
        &mut self,
        _resources: &mut Resources,
        _button: &MouseButton,
        _button_state: &ElementState,
    ) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_key(&mut self, _resources: &mut Resources, _input: KeyboardInput) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_gamepad_event(
        &mut self,
        _resources: &mut Resources,
        _event: GilrsEvent,
    ) -> Result<Transition> {
        Ok(Transition::None)
    }

    fn on_event(&mut self, _resources: &mut Resources, _event: &Event<()>) -> Result<Transition> {
        Ok(Transition::None)
    }
}

pub enum Transition {
    None,
    Pop,
    Push(Box<dyn State>),
    Switch(Box<dyn State>),
    Quit,
}

pub struct StateMachine {
    running: bool,
    states: Vec<Box<dyn State>>,
}

impl StateMachine {
    pub fn new(initial_state: impl State + 'static) -> Self {
        Self {
            running: false,
            states: vec![Box::new(initial_state)],
        }
    }

    pub fn current_state(&mut self) -> Result<&mut Box<(dyn State + 'static)>> {
        self.states
            .last_mut()
            .context("Tried to access state in state machine with no states present!")
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn start(&mut self, resources: &mut Resources) -> Result<()> {
        if !self.running {
            let state = self.current_state()?;
            state.on_start(resources)?;
            self.running = true;
        }
        Ok(())
    }

    pub fn handle_event(&mut self, resources: &mut Resources, event: &Event<()>) -> Result<()> {
        if self.running {
            let transition = match self.states.last_mut() {
                Some(state) => state.on_event(resources, &event)?,
                None => Transition::None,
            };
            self.transition(transition, resources)?;
        }
        Ok(())
    }

    pub fn update(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            let transition = match self.states.last_mut() {
                Some(state) => state.update(resources)?,
                None => Transition::None,
            };
            self.transition(transition, resources)?;
        }
        Ok(())
    }

    pub fn transition(&mut self, request: Transition, resources: &mut Resources) -> Result<()> {
        if self.running {
            match request {
                Transition::None => (),
                Transition::Pop => self.pop(resources)?,
                Transition::Push(state) => self.push(state, resources)?,
                Transition::Switch(state) => self.switch(state, resources)?,
                Transition::Quit => self.stop(resources)?,
            }
        }
        Ok(())
    }

    fn switch(&mut self, state: Box<dyn State>, resources: &mut Resources) -> Result<()> {
        if self.running {
            if let Some(mut state) = self.states.pop() {
                state.on_stop(resources)?;
            }
            self.states.push(state);
            let new_state = self.current_state()?;
            new_state.on_start(resources)?;
        }
        Ok(())
    }

    fn push(&mut self, state: Box<dyn State>, resources: &mut Resources) -> Result<()> {
        if self.running {
            if let Ok(state) = self.current_state() {
                state.on_pause(resources)?;
            }
            self.states.push(state);
            let new_state = self.current_state()?;
            new_state.on_start(resources)?;
        }
        Ok(())
    }

    fn pop(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            if let Some(mut state) = self.states.pop() {
                state.on_stop(resources)?;
            }
            if let Some(state) = self.states.last_mut() {
                state.on_resume(resources)?;
            } else {
                self.running = false;
            }
        }
        Ok(())
    }

    pub fn stop(&mut self, resources: &mut Resources) -> Result<()> {
        if self.running {
            while let Some(mut state) = self.states.pop() {
                state.on_stop(resources)?;
            }
            self.running = false;
        }
        Ok(())
    }
}
