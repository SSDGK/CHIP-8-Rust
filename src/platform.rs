use crate::audio::Beeper;
use crate::cpu::{DISPLAY_HEIGHT, DISPLAY_WIDTH, Platform};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;

/// 桌面平台实现：minifb 窗口 + rodio 蜂鸣器
pub struct DesktopPlatform {
    pub window: Window,
    beeper: Beeper,
    buffer: Vec<u32>,
}

impl DesktopPlatform {
    /// 创建窗口并初始化音频
    pub fn new(title: &str, scale: usize) -> Self {
        let width = DISPLAY_WIDTH * scale;
        let height = DISPLAY_HEIGHT * scale;
        let mut window = Window::new(title, width, height, WindowOptions::default())
            .unwrap_or_else(|e| panic!("{}", e));
        window.limit_update_rate(Some(Duration::from_micros(16600)));

        Self {
            window,
            beeper: Beeper::new(),
            buffer: vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
        }
    }
}

impl Platform for DesktopPlatform {
    fn poll_keys(&self) -> [bool; 16] {
        let mut keys = [false; 16];
        for key in self.window.get_keys() {
            match key {
                Key::X => keys[0x0] = true,
                Key::Key1 => keys[0x1] = true,
                Key::Key2 => keys[0x2] = true,
                Key::Key3 => keys[0x3] = true,
                Key::Q => keys[0x4] = true,
                Key::W => keys[0x5] = true,
                Key::E => keys[0x6] = true,
                Key::A => keys[0x7] = true,
                Key::S => keys[0x8] = true,
                Key::D => keys[0x9] = true,
                Key::Z => keys[0xA] = true,
                Key::C => keys[0xB] = true,
                Key::Key4 => keys[0xC] = true,
                Key::R => keys[0xD] = true,
                Key::F => keys[0xE] = true,
                Key::V => keys[0xF] = true,
                _ => (),
            }
        }
        keys
    }

    fn render(&mut self, display: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
        for (dest, &src) in self.buffer.iter_mut().zip(display.iter()) {
            *dest = if src { 0xFFFFFF } else { 0x000000 };
        }
        self.window
            .update_with_buffer(&self.buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .unwrap();
    }

    fn beep(&self, on: bool) {
        self.beeper.set_playing(on);
    }
}
