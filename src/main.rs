#![allow(dead_code)]

use std::{time::{Instant, Duration}};
use emulator::{machine::Machine, comp_mode::{CompBuilder, CompatibilityMode}, keys::Keys};
use pixels::{PixelsBuilder, SurfaceTexture, Pixels};
use winit::{window::{Window, WindowBuilder}, event_loop::{EventLoop, ControlFlow}, platform::run_return::EventLoopExtRunReturn, event::{VirtualKeyCode, KeyboardInput, ElementState}};
use rand::prelude::*;

mod emulator;

const PROGRAM_START: usize = 0x200;
const PROGRAM: &str = "./programs/rockto.ch8";
const INSTRUCTIONS_PER_FRAME: usize = 10;

fn main() {
    let (mut state, mut ev_loop) = State::new();

    ev_loop.run_return(|ev, _, cf| {
        use winit::event::Event;
        use winit::event::WindowEvent;
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => state.running = false,
                WindowEvent::Resized(size) => state.resize(size.width, size.height),
                WindowEvent::KeyboardInput { input, .. } => state.key_input(input),
                _ => ()
            }
            Event::MainEventsCleared => {
                state.update();
                state.render();
                state.configure_cf(cf);
            }
            _ => (),
        }
    });
}


struct State {
    comp: CompatibilityMode,
    machine: Machine,
    next_decrement: Instant,
    decrement_time: Duration,
    window: Window,
    running: bool,
    pixels: Pixels,
    keys: Keys,
}
impl State {
    fn new() -> (Self, EventLoop<()>) {
        let program = std::fs::read(PROGRAM).unwrap();
        let comp = CompBuilder::superchip_preset()
            .build();

        let mut machine = Machine::new(thread_rng().gen());
        machine.init_instruction_pointer(PROGRAM_START as u16);
        machine.load_sprites();
        machine.load_program(&program, PROGRAM_START);

        let next_decrement = Instant::now();
        let decrement_time = Duration::from_secs_f64(1.0 / 60.0);

        let ev_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&ev_loop)
            .unwrap();

        let surface_texture = SurfaceTexture::new(window.inner_size().width, window.inner_size().height, &window);
        let pixels = PixelsBuilder::new(128, 64, surface_texture)
            .build().unwrap();

        let ret = Self {
            comp,
            machine,
            next_decrement,
            decrement_time,
            window,
            running: true,
            pixels,
            keys: Keys::new(),
        };

        


        (ret, ev_loop)
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.pixels.resize_surface(width, height).unwrap();
    }
    fn configure_cf(&self, cf: &mut ControlFlow) {
        if self.running {
            *cf = ControlFlow::Poll;
        }
        else {
            *cf = ControlFlow::Exit;
        }
    }

    fn key_input(&mut self, i: KeyboardInput) {
        if let Some(code) = i.virtual_keycode {
            for &(key, val) in KEY_MAP {
                if key == code {
                    let is_down = i.state == ElementState::Pressed;
                    self.keys.set_key(val, is_down);
                }
            }
        }
    }
    fn update(&mut self) {
        let now = Instant::now();
        while self.next_decrement <= now {
            self.machine.decrement_counters();
            self.next_decrement += self.decrement_time;
        }

        for _ in 0..INSTRUCTIONS_PER_FRAME {
            self.machine.decode_and_execute(&self.comp, &self.keys);
        }
    }

    fn render(&mut self) {
        self.machine.screen().render_to_pixel_buffer(self.pixels.get_frame_mut());
        self.pixels.render().unwrap();
    }
}



static KEY_MAP: &[(VirtualKeyCode, u8)] = &[
    (VirtualKeyCode::Key1, 0x1),
    (VirtualKeyCode::Key2, 0x2),
    (VirtualKeyCode::Key3, 0x3),
    (VirtualKeyCode::Key4, 0xC),

    (VirtualKeyCode::Q, 0x4),
    (VirtualKeyCode::W, 0x5),
    (VirtualKeyCode::E, 0x6),
    (VirtualKeyCode::R, 0xD),

    (VirtualKeyCode::A, 0x7),
    (VirtualKeyCode::S, 0x8),
    (VirtualKeyCode::D, 0x9),
    (VirtualKeyCode::F, 0xE),

    (VirtualKeyCode::Z, 0xA),
    (VirtualKeyCode::X, 0x0),
    (VirtualKeyCode::C, 0xB),
    (VirtualKeyCode::V, 0xF),
];
