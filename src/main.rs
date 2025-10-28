mod entities;
mod models;
mod systems;

use std::thread::sleep;
use std::time::Duration;
use crate::entities::spawn_goblin;
use crate::models::{Player, Position, Renderable};
use crate::systems::{AiSystem, InputSystem, SystemFunc};
use doryen_rs::{App, AppOptions, DoryenApi, Engine, UpdateEvent};
use hecs::World;
// // this part makes it possible to compile to wasm32 target
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;
// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen(start)]
// pub fn main_js() -> Result<(), JsValue> {
//     main();
//     Ok(())
// }

/*
Apart from the basic real-time walking, this example shows how screenshots can be captured in-game.
Because it uses UpdateEvent, any combination of keys can be specified to activate it.
*/

const CONSOLE_WIDTH: u32 = 80;
const CONSOLE_HEIGHT: u32 = 45;

// type System = Box<dyn FnMut(&mut World)>;

struct MyRoguelike {
    world: World,
    systems: Vec<Box<dyn SystemFunc>>,
}

impl Engine for MyRoguelike {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        api.con().register_color("white", (255, 255, 255, 255));
        api.con().register_color("red", (255, 92, 92, 255));
        api.con().register_color("blue", (192, 192, 255, 255));

        self.world.spawn((
            Player {},
            Position::new((CONSOLE_WIDTH / 2) as isize, (CONSOLE_HEIGHT / 2) as isize),
            Renderable {
                glyph: '@',
                color: (255, 92, 92, 255),
            },
        ));

        spawn_goblin(
            &mut self.world,
            5,
            (5, 10),
            (CONSOLE_WIDTH as usize - 2, CONSOLE_HEIGHT as usize - 2),
        );
    }
    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        // capture the screen
        // if input.key("ControlLeft") && input.key_pressed("KeyS") {
        //     self.screenshot_idx += 1;
        //     return Some(UpdateEvent::Capture(format!(
        //         "screenshot_{:03}.png",
        //         self.screenshot_idx
        //     )));
        // }

        for system in &mut self.systems {
            system.call(&mut self.world, api)
        }
        // sleep(Duration::from_millis(250));

        None
    }
    fn render(&mut self, api: &mut dyn DoryenApi) {
        let con = api.con();
        con.rectangle(
            0,
            0,
            CONSOLE_WIDTH,
            CONSOLE_HEIGHT,
            Some((128, 128, 128, 255)),
            Some((0, 0, 0, 255)),
            Some('.' as u16),
        );
        // con.ascii(self.player_pos.0, self.player_pos.1, '@' as u16);
        // con.fore(self.player_pos.0, self.player_pos.1, (255, 255, 255, 255));

        for (_id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            con.ascii(pos.x as i32, pos.y as i32, render.glyph as u16);
            con.fore(pos.x as i32, pos.y as i32, render.color);
        }
    }
}

impl MyRoguelike {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            systems: vec![Box::new(InputSystem {}), Box::new(AiSystem {})],
        }
    }
}

fn main() {
    // here are all the available options.
    // better practise is to use default values (see other examples)
    let options = AppOptions {
        console_width: CONSOLE_WIDTH,
        console_height: CONSOLE_HEIGHT,
        screen_width: CONSOLE_WIDTH * 8,
        screen_height: CONSOLE_HEIGHT * 8,
        window_title: "my roguelike".to_owned(),
        font_path: "terminal_8x8.png".to_owned(),
        vsync: true,
        fullscreen: false,
        show_cursor: true,
        resizable: false,
        intercept_close_request: false,
        max_fps: 12,
    };
    let mut app = App::new(options);

    app.set_engine(Box::new(MyRoguelike::new()));

    app.run();
}
