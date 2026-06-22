use crate::instruction::Instruction;
use std::fs;

pub const MEMORY_SIZE: usize = 4096;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const PC_START_ADDR: u16 = 0x200;

/// CHIP-8 外设抽象接口
pub trait Platform {
    /// 读取 16 键键盘状态
    fn poll_keys(&self) -> [bool; 16];
    /// 渲染显示缓冲区
    fn render(&mut self, display: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
    /// 控制蜂鸣器开关
    fn beep(&self, on: bool);
}

/// CHIP-8 虚拟机核心结构体
pub struct Chip8<P: Platform> {
    pub memory: [u8; MEMORY_SIZE],                       // 4KB 内存
    pub v: [u8; 16],                                     // 通用寄存器 V0-VF
    pub i: u16,                                          // 地址索引寄存器
    pub pc: u16,                                         // 程序计数器
    pub stack: [u16; 16],                                // 调用栈
    pub sp: u8,                                          // 栈指针
    pub delay_timer: u8,                                 // 延迟定时器 (60Hz)
    pub sound_timer: u8,                                 // 声音定时器 (60Hz)
    pub keypad: [bool; 16],                              // 16 键键盘状态
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT], // 64x32 像素显示缓冲区
    pub platform: P,                                     // 外设平台
}

impl<P: Platform> Chip8<P> {
    /// 创建并初始化虚拟机，加载内置字模到内存 0x050 处
    pub fn new(platform: P) -> Self {
        let mut new_cpu = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            pc: PC_START_ADDR,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            platform,
        };

        new_cpu.memory[0x050..0x0A0].copy_from_slice(&FONTSET);
        new_cpu
    }

    /// 将 ROM 文件加载到内存 0x200 处
    pub fn load_rom(&mut self, filename: &str) -> Result<(), String> {
        let rom_data = fs::read(filename).map_err(|_| String::from("未找到文件"))?;

        if rom_data.len() > (MEMORY_SIZE - PC_START_ADDR as usize) {
            return Err(String::from("文件过大"));
        }

        let start_addr = PC_START_ADDR as usize;
        let end_addr = start_addr + rom_data.len();

        self.memory[start_addr..end_addr].copy_from_slice(&rom_data);
        Ok(())
    }

    /// 执行一个 CPU 周期：取指 → 译码 → 执行
    pub fn tick(&mut self) -> Result<(), String> {
        self.keypad = self.platform.poll_keys();

        let pc = self.pc as usize;
        if pc + 1 >= MEMORY_SIZE {
            return Err(format!("致命错误：PC 指针越界 (0x{:04X})", pc));
        }

        let opcode = (self.memory[pc] as u16) << 8 | (self.memory[pc + 1] as u16);
        self.pc += 2;

        let inst = Instruction::decode(opcode);
        self.execute(inst)
    }

    /// 根据译码结果执行对应指令
    fn execute(&mut self, inst: Instruction) -> Result<(), String> {
        match inst {
            Instruction::ClearScreen => {
                self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
            }
            Instruction::Return => {
                if self.sp == 0 {
                    return Err(String::from("致命错误：堆栈下溢！"));
                }
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            Instruction::Jump(addr) => self.pc = addr,
            Instruction::Call(addr) => {
                if self.sp >= 16 {
                    return Err(String::from("致命错误：堆栈溢出！"));
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }
            Instruction::SkipEqVxN(x, val) => {
                if self.v[x as usize] == val {
                    self.pc += 2;
                }
            }
            Instruction::SkipNeqVxN(x, val) => {
                if self.v[x as usize] != val {
                    self.pc += 2;
                }
            }
            Instruction::SkipEqVxVy(x, y) => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::SetVx(x, val) => self.v[x as usize] = val,
            Instruction::SetI(addr) => self.i = addr,
            Instruction::AddVx(x, val) => {
                self.v[x as usize] = self.v[x as usize].wrapping_add(val);
            }
            Instruction::SetVxVy(x, y) => self.v[x as usize] = self.v[y as usize],
            Instruction::OrVxVy(x, y) => self.v[x as usize] |= self.v[y as usize],
            Instruction::AndVxVy(x, y) => self.v[x as usize] &= self.v[y as usize],
            Instruction::XorVxVy(x, y) => self.v[x as usize] ^= self.v[y as usize],
            Instruction::AddVxVy(x, y) => {
                let (res, overflow) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = if overflow { 1 } else { 0 };
            }
            Instruction::SubVxVy(x, y) => {
                let (res, overflow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = if !overflow { 1 } else { 0 };
            }
            Instruction::ShrVx(x, _y) => {
                let lsb = self.v[x as usize] & 1;
                self.v[x as usize] >>= 1;
                self.v[0xF] = lsb;
            }
            Instruction::SubnVxVy(x, y) => {
                let (res, overflow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = if !overflow { 1 } else { 0 };
            }
            Instruction::ShlVx(x, _y) => {
                let msb = (self.v[x as usize] >> 7) & 1;
                self.v[x as usize] <<= 1;
                self.v[0xF] = msb;
            }
            Instruction::LdDTVx(x) => self.v[x as usize] = self.delay_timer,
            Instruction::LdKeyVx(x) => {
                let mut pressed = false;
                for (i, &key) in self.keypad.iter().enumerate() {
                    if key {
                        self.v[x as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            Instruction::SetDTVx(x) => self.delay_timer = self.v[x as usize],
            Instruction::SetSTVx(x) => self.sound_timer = self.v[x as usize],
            Instruction::AddIVx(x) => {
                self.i = self.i.wrapping_add(self.v[x as usize] as u16);
            }
            Instruction::LdSpriteI(x) => {
                let digit = self.v[x as usize] as u16;
                self.i = 0x050 + (digit * 5);
            }
            Instruction::BcdVx(x) => {
                let val = self.v[x as usize];
                self.memory[self.i as usize] = val / 100;
                self.memory[(self.i + 1) as usize] = (val / 10) % 10;
                self.memory[(self.i + 2) as usize] = val % 10;
            }
            Instruction::StoreRegs(x) => {
                for idx in 0..=x as usize {
                    self.memory[self.i as usize + idx] = self.v[idx];
                }
            }
            Instruction::LoadRegs(x) => {
                for idx in 0..=x as usize {
                    self.v[idx] = self.memory[self.i as usize + idx];
                }
            }
            Instruction::SkipNeqVxVy(x, y) => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::JumpV0(addr) => self.pc = addr + (self.v[0] as u16),
            Instruction::Rand(x, val) => {
                let random_byte: u8 = rand::random();
                self.v[x as usize] = random_byte & val;
            }
            Instruction::SkipKey(x) => {
                let key = self.v[x as usize];
                if self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            Instruction::SkipNotKey(x) => {
                let key = self.v[x as usize];
                if !self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            Instruction::Draw(x, y, n) => {
                let start_x = self.v[x as usize] as usize % DISPLAY_WIDTH;
                let start_y = self.v[y as usize] as usize % DISPLAY_HEIGHT;

                self.v[0xF] = 0;

                for row in 0..n {
                    let y_coord = start_y + row as usize;
                    if y_coord >= DISPLAY_HEIGHT {
                        break;
                    }

                    let sprite_byte = self.memory[(self.i + row as u16) as usize];

                    for col in 0..8 {
                        let x_coord = start_x + col;
                        if x_coord >= DISPLAY_WIDTH {
                            break;
                        }

                        let sprite_pixel = (sprite_byte & (0x80 >> col)) != 0;

                        if sprite_pixel {
                            let screen_idx = y_coord * DISPLAY_WIDTH + x_coord;
                            if self.display[screen_idx] {
                                self.v[0xF] = 1;
                            }
                            self.display[screen_idx] ^= true;
                        }
                    }
                }
            }
            Instruction::Unknown(op) => {
                return Err(format!(
                    "未实现或未知的指令: {:04X} 位于地址 0x{:04X}",
                    op,
                    self.pc - 2
                ));
            }
        }
        Ok(())
    }

    /// 更新定时器并控制蜂鸣器（每帧调用一次，60Hz）
    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        self.platform.beep(self.sound_timer > 0);
    }

    /// 将显示缓冲区提交给平台渲染
    pub fn render(&mut self) {
        self.platform.render(&self.display);
    }
}

/// 内置 4x5 字模数据 (0-9, A-F)，每个字符 5 字节
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction;

    struct TestPlatform {
        keys: [bool; 16],
    }

    impl Platform for TestPlatform {
        fn poll_keys(&self) -> [bool; 16] {
            self.keys
        }
        fn render(&mut self, _display: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {}
        fn beep(&self, _on: bool) {}
    }

    fn make_cpu() -> Chip8<TestPlatform> {
        Chip8::new(TestPlatform { keys: [false; 16] })
    }

    #[test]
    fn test_initial_state() {
        let cpu = make_cpu();
        assert_eq!(cpu.pc, PC_START_ADDR);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_jump_instruction() {
        let mut cpu = make_cpu();
        cpu.execute(Instruction::Jump(0x345)).unwrap();
        assert_eq!(cpu.pc, 0x345);
    }

    #[test]
    fn test_add_vx_overflow() {
        let mut cpu = make_cpu();
        cpu.v[0] = 250;
        cpu.execute(Instruction::AddVx(0, 10)).unwrap();
        assert_eq!(cpu.v[0], 4);
    }

    #[test]
    fn test_clear_screen() {
        let mut cpu = make_cpu();
        cpu.display[0] = true;
        cpu.display[2047] = true;
        cpu.execute(Instruction::ClearScreen).unwrap();
        assert!(!cpu.display.contains(&true));
    }

    #[test]
    fn test_call_and_return() {
        let mut cpu = make_cpu();
        cpu.pc = PC_START_ADDR;
        cpu.execute(Instruction::Call(0x300)).unwrap();
        assert_eq!(cpu.pc, 0x300);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], PC_START_ADDR);

        cpu.execute(Instruction::Return).unwrap();
        assert_eq!(cpu.pc, PC_START_ADDR);
        assert_eq!(cpu.sp, 0);
    }
}
