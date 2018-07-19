extern crate hp;
use hp::client_lib::HostilePlanetsClient;

extern crate chrono;
extern crate piston_window;
extern crate timer;

// use chrono::*;
use piston_window::*;
use std::sync::{Arc, Mutex};
use std::{thread, time};
// use std::time;
// use timer::*;

// extern crate graphics;
// extern crate glutin_window;
// extern crate opengl_graphics;

// use piston::window::WindowSettings;
// use piston::event_loop::*;
// use piston::input::*;
// use glutin_window::GlutinWindow as Window;
// use opengl_graphics::{ GlGraphics, OpenGL };

// pub struct App {
//     gl: GlGraphics, // OpenGL drawing backend.
//     rotation: f64   // Rotation for the square.
// }

// impl App {
//     fn render(&mut self, args: &RenderArgs) {
//         use graphics::*;

//         const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
//         const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

//         let square = rectangle::square(0.0, 0.0, 50.0);
//         let rotation = self.rotation;
//         let (x, y) = ((args.width / 2) as f64,
//                       (args.height / 2) as f64);

//         self.gl.draw(args.viewport(), |c, gl| {
//             // Clear the screen.
//             clear(GREEN, gl);

//             let transform = c.transform.trans(x, y)
//                                        .rot_rad(rotation)
//                                        .trans(-25.0, -25.0);

//             // Draw a box rotating around the middle of the screen.
//             rectangle(RED, square, transform, gl);
//         });
//     }

//     fn update(&mut self, args: &UpdateArgs) {
//         // Rotate 2 radians per second.
//         self.rotation += 2.0 * args.dt;
//     }
// }

fn connect(c: &mut HostilePlanetsClient) {
    match c.connect() {
        Ok(stream) => {
            c.server_con = Some(stream);
        }

        Err(e) => {
            println!(
                "failed connecting to server: {}, retrying in 10 seconds...",
                e
            );
            thread::sleep(time::Duration::from_millis(1000 * 10));
            connect(c);
        }
    }
}

fn main() {
    let c = Arc::new(Mutex::new(HostilePlanetsClient::new("conf.toml")));
    {
        let c = c.clone();
        thread::spawn(move || {
            connect(&mut *c.lock().unwrap());
        });
    }

    println!(
        "The client is instantiated, and attempting to connect to the server: {:?}",
        *c.lock().unwrap()
    );

    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();

    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            clear([1.0; 4], graphics);
            rectangle(
                [1.0, 0.0, 0.0, 1.0], // red
                [0.0, 0.0, 100.0, 100.0],
                context.transform,
                graphics,
            );
        });
    }

    // // Change this to OpenGL::V2_1 if not working.
    // let opengl = OpenGL::V3_2;

    // // Create an Glutin window.
    // let mut window: Window = WindowSettings::new(
    //         "spinning-square",
    //         [200, 200]
    //     )
    //     .opengl(opengl)
    //     .exit_on_esc(true)
    //     .build()
    //     .unwrap();

    // // Create a new game and run it.
    // let mut app = App {
    //     gl: GlGraphics::new(opengl),
    //     rotation: 0.0
    // };

    // let mut events = Events::new(EventSettings::new());
    // while let Some(e) = events.next(&mut window) {
    //     if let Some(r) = e.render_args() {
    //         app.render(&r);
    //     }

    //     if let Some(u) = e.update_args() {
    //         app.update(&u);
    //     }
    // }
}
