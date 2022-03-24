use std::time::Duration;

use headless_webview::{prelude::*, types::WindowSize};
use headless_webview_engine;

// Linux: tested on X11 (gnome, ubuntu)
#[test]
fn test_texture() {
    let (width, mut height) = (600, 400);
    let window = headless_webview_engine::headless()
        .with_inner_size(WindowSize::new(width, height))
        //.with_transparent(true)
        .build()
        .unwrap();

    let mut webview = WebviewBuilder::new(window)
        .unwrap()
        //.with_transparent(true)
        .with_html(
            "<body style=\"margin: 10px; background: rgba(255, 255, 0, 0.5);\">hello world!</body>",
        )
        .unwrap()
        .build()
        .unwrap();

    let version = webview.version().unwrap();

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    assert!(
        version.starts_with("webkit2gtk-v"),
        "{:?} should start with webview2-v",
        version
    );

    #[cfg(target_os = "windows")]
    assert!(
        version.starts_with("webview2-v"),
        "{:?} should start with webview2-v",
        version
    );

    webview.tick(types::TickMode::periodic_60hz(Duration::from_millis(500)));

    let texture = webview.get_texture().unwrap().unwrap();
    let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_vec(
        texture.width as u32,
        texture.height as u32,
        texture.data.clone(),
    )
    .unwrap();
    img.save("test1.png").unwrap();

    assert_eq!(texture.width, width);
    assert_eq!(texture.height, height);
    assert_eq!(texture.format, types::TextureFormat::Rgba8);
    assert_eq!(
        texture.data.len(),
        width as usize * height as usize * texture.format.n_channels()
    );

    assert_eq!(&texture.data[..4], &[255, 255, 127, 255]);

    // test resize
    let (width, mut height) = (800, 600);
    webview.resize(WindowSize::new(width, height)).unwrap();

    webview.tick(types::TickMode::periodic_60hz(Duration::from_millis(500)));

    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, width);
    assert_eq!(texture.height, height);
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

    img.save("test2.png").unwrap();

    // bottom right corner must have data now
    assert_eq!(
        &texture.data[texture.data.len() - 4..],
        &[255, 255, 127, 255]
    );

    // test resize back
    let (width, height) = (400, 300);
    webview.resize(WindowSize::new(width, height)).unwrap();

    webview.tick(types::TickMode::periodic_60hz(Duration::from_millis(500)));

    //assert_eq!(webview.window().inner_size(), (width, height));
    let texture = webview.get_texture().unwrap().unwrap();
    assert_eq!(texture.width, width);
    assert_eq!(texture.height, height);
    assert_eq!(
        texture.data.len(),
        width as usize * height as usize * texture.format.n_channels()
    );

    // bottom right corner must have data now
    assert_eq!(
        &texture.data[texture.data.len() - 4..],
        &[255, 255, 127, 255]
    );
}
