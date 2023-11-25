//use image::{ImageBuffer, Rgba};
use std::time::Duration;

use headless_webview::{
    self,
    types::{self, WindowSize},
    webview::Color,
    EngineWebview, WebviewBuilder,
};

pub fn main() {
    let window = headless_webview_engine::windowed()
        .with_inner_size(WindowSize::new(600, 400))
        .with_transparent(true)
        .build()
        .unwrap();

    let mut webview = WebviewBuilder::new(window)
        .unwrap()
        .with_color(Color::new(1., 1., 1., 0.))
        .with_html(
            "<body style=\"margin: 10px; background: rgba(255, 255, 0, 0.5);\">hello world!</body>",
        )
        .unwrap()
        .build()
        .unwrap();

    webview.tick(types::TickMode::WaitFor(Duration::from_millis(300)));

    webview.send_mouse_event(types::MouseEvent {
        button: types::MouseButton::Left,
        state: types::ElementState::Pressed,
        position: types::Vec2::new(20., 30.),
    });

    webview.send_mouse_event(types::MouseEvent {
        button: types::MouseButton::Left,
        state: types::ElementState::Released,
        position: types::Vec2::new(20., 30.),
    });

    webview.send_keyboard_input(types::KeyboardInput {
        state: types::ElementState::Pressed,
    });

    webview.tick(types::TickMode::WaitFor(Duration::from_millis(300)));

    /*
    let mut frame = 0;
    let texture = webview.get_texture().unwrap().unwrap();

    println!(
        "TEXTURE {:?} {} {} {:?}",
        texture.format,
        texture.width,
        texture.height,
        &texture.data[..4]
    );
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
    ImageBuffer::from_vec(texture.width as u32, texture.height as u32, texture.data).unwrap();

    img.save(format!("test{}.png", frame)).unwrap();
    */

    loop {
        webview.tick(types::TickMode::WaitFor(Duration::from_millis(16)));
        if let Ok(Some(texture)) = webview.get_texture() {
            println!("TEXTURE {:?}", &texture.data[..4]);
            // let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            //     ImageBuffer::from_vec(texture.width as u32, texture.height as u32, texture.data)
            //         .unwrap();

            // img.save(format!("test{}.png", frame)).unwrap();
        }
        //frame += 1;
    }
}
