use std::time::Duration;

use headless_webview::{prelude::*, types::WindowSize, webview::Color};
use headless_webview_engine;

/// Linux: tested on X11 (gnome)
#[test]
fn test_windowed_transparent() {
    let (width, height) = (600, 400);
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

    webview.tick(types::TickMode::periodic_60hz(Duration::from_millis(500)));

    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, width);
    assert_eq!(texture.height, height);
    assert_eq!(texture.format, types::TextureFormat::Rgba8);
    assert_eq!(
        texture.data.len(),
        width as usize * height as usize * texture.format.n_channels()
    );

    let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_vec(
        texture.width as u32,
        texture.height as u32,
        texture.data.clone(),
    )
    .unwrap();
    img.save("window_transparent1.png").unwrap();

    assert_eq!(&texture.data[..4], &[255, 255, 0, 128]);

    // test resize
    webview.resize(WindowSize::new(800, 600)).unwrap();

    webview.tick(types::TickMode::periodic_60hz(Duration::from_millis(50)));
    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, 800);
    assert_eq!(texture.height, 600);
    assert_eq!(texture.data.len(), 800 * 600 * texture.format.n_channels());

    // bottom right corner must have data now
    assert_eq!(&texture.data[texture.data.len() - 4..], &[255, 255, 0, 128]);

    // test resize back
    webview.resize(WindowSize::new(400, 300)).unwrap();

    webview.tick(types::TickMode::periodic_60hz(Duration::from_millis(50)));
    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, 400);
    assert_eq!(texture.height, 300);
    assert_eq!(texture.data.len(), 400 * 300 * texture.format.n_channels());

    // bottom right corner must have data now
    assert_eq!(&texture.data[texture.data.len() - 4..], &[255, 255, 0, 128]);
}
