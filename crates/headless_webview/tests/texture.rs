use std::time::Duration;

use headless_webview::{prelude::*, types::WindowSize};

#[test]
pub fn test_texture() {
    let window = engines::dummy().build().unwrap();
    let mut webview = WebviewBuilder::new(window).unwrap().build().unwrap();

    assert_eq!(webview.version().unwrap(), "dummy-v0.0.1");

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

    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, 800);
    assert_eq!(texture.height, 600);
    assert_eq!(&texture.data[..4], &[50, 180, 50, 255]);

    webview.resize(WindowSize::new(600, 400)).unwrap();
    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, 600);
    assert_eq!(texture.height, 400);
}
