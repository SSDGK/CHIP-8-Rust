mod audio;
mod cpu;
mod instruction;
mod platform;

use cpu::Chip8;
use minifb::Key;
use platform::DesktopPlatform;

// 像素放大倍数
const SCALE: usize = 10;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("用法: cargo run -- <游戏路径>");
        return;
    }

    let platform = DesktopPlatform::new("CHIP-8", SCALE);
    let mut chip8 = Chip8::new(platform);

    if let Err(e) = chip8.load_rom(&args[1]) {
        eprintln!("加载失败: {}", e);
        return;
    }

    let mut halted = false;

    while chip8.platform.window.is_open() && !chip8.platform.window.is_key_down(Key::Escape) {
        if !halted {
            for _ in 0..10 {
                if let Err(e) = chip8.tick() {
                    println!("CPU 停机: {}", e);
                    halted = true;
                    break;
                }
            }
            chip8.update_timers();
        }
        chip8.render();
    }
}
