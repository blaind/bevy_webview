use std::time::Duration;

/// Represents a mouse event at a specific position
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// Button that triggered the event
    pub button: MouseButton,

    /// Button state
    pub state: ElementState,

    /// Position where mouse cursor was when the event occured
    pub position: Vec2,
}

/// State for input event
#[derive(Debug, Clone)]
pub enum ElementState {
    Pressed,
    Released,
}

/// Keyboard event
#[derive(Debug, Clone)]
pub struct KeyboardInput {
    pub state: ElementState,
    // TODO: add key
}

/// A 2D vector with x and y
#[derive(Debug, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Mouse button identifier
#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16), // FIXME: unsafe
}

/// Webview output texture
#[derive(Debug, Clone)]
pub struct Texture {
    /// Width of the texture, in pixels
    pub width: u32,

    /// Height of the texture, in pixels
    pub height: u32,

    /// Texture pixel format
    pub format: TextureFormat,

    /// Byte data of the texture
    pub data: Vec<u8>,
}

impl Texture {
    /// Expected bytesize of the texture data
    pub fn buffer_size(&self) -> usize {
        self.width as usize * self.height as usize * self.format.n_channels()
    }
}

/// Texture format
#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    /// 8-bit RGB
    Rgb8,

    /// 8-bit RGB with alpha channel
    Rgba8,
}

impl TextureFormat {
    /// Number of channels in the texture
    pub fn n_channels(&self) -> usize {
        match self {
            TextureFormat::Rgb8 => 3,
            TextureFormat::Rgba8 => 4,
        }
    }
}

/// Tick variant
pub enum TickMode {
    /// Non-blocking tick, process events and return
    Immediate,

    /// Blocking tick, process the events and wait until `Duration` is spent in total
    WaitFor(Duration),

    /// Tick periodically, with limited total execution time
    PeriodicWait(PeriodicWait),
}

impl TickMode {
    pub fn wait(duration: Duration) -> Self {
        Self::WaitFor(duration)
    }

    pub fn periodic_60hz(duration: Duration) -> Self {
        Self::periodic(
            duration,
            Duration::from_micros(((1000.0 / 60.) * 1000.0) as u64),
        )
    }

    pub fn periodic(duration: Duration, tick_interval: Duration) -> Self {
        Self::PeriodicWait(PeriodicWait {
            duration,
            tick_interval,
        })
    }
}

/// Tick periodiocally, with limited total execution time
pub struct PeriodicWait {
    /// Total execution time
    pub duration: Duration,

    /// Tick every `tick_interval` until `duration` is reached
    pub tick_interval: Duration,
}

/// Window size
#[derive(Clone, Debug)]
pub struct WindowSize {
    /// Width of the window, in pixels
    pub width: u32,

    /// Height of the window, in pixels
    pub height: u32,
}

impl WindowSize {
    /// Construct new
    pub fn new(width: u32, height: u32) -> Self {
        WindowSize { width, height }
    }
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
        }
    }
}
