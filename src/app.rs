use chrono::{Local, Timelike};

use crate::render_primitives::RenderPrimitives;

pub struct App {
    width: f32,
    height: f32,
    need_redraw: bool,
    show_state: ShowState,
    pub primitives: Vec<RenderPrimitives>,
}

#[derive(PartialEq)]
struct ShowState {
    hour: u32,
    minute: u32,
}

impl App {
    pub fn new(width: f32, height: f32) -> App {
        let mut app = App {
            width,
            height,
            need_redraw: true,
            show_state: App::update_show_state(),
            primitives: Vec::with_capacity(0),
        };
        app.invalidate();
        app
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.invalidate();
    }

    pub fn update_and_check_need_redraw(&mut self) -> bool {
        // If true, redraw is needed.
        let next_show_state = App::update_show_state();
        if self.show_state != next_show_state {
            self.show_state = next_show_state;
            self.invalidate();
        }

        let result = self.need_redraw;
        if result {
            self.need_redraw = false;
        }
        result
    }

    fn update_show_state() -> ShowState {
        let time = Local::now();
        ShowState {
            hour: time.hour(),
            minute: time.minute(),
        }
    }

    fn invalidate(&mut self) {
        self.need_redraw = true;
        self.primitives = vec![
            RenderPrimitives::Clear {
                color: (0x000000, 0.0),
            },
            RenderPrimitives::VertCenteredText {
                text: format!("{:02}:{:02}", self.show_state.hour, self.show_state.minute),
                rect: (0.0, 0.0, self.width, self.height),
                size_font: 54.0,
                size_space: 1.0,
                weight_stroke: 1.0,
                color_fill: (0xFFFFFF, 1.0),
                color_stroke: (0x000000, 1.0),
            },
        ];
    }
}
