/// CHIP-8 指令集
#[derive(Debug)]
pub enum Instruction {
    ClearScreen,         // 00E0
    Return,              // 00EE
    Jump(u16),           // 1NNN
    Call(u16),           // 2NNN
    SkipEqVxN(u8, u8),   // 3XNN: Vx == NN 时跳过
    SkipNeqVxN(u8, u8),  // 4XNN: Vx != NN 时跳过
    SkipEqVxVy(u8, u8),  // 5XY0: Vx == Vy 时跳过
    SetVx(u8, u8),       // 6XNN: Vx = NN
    AddVx(u8, u8),       // 7XNN: Vx += NN
    SetVxVy(u8, u8),     // 8XY0: Vx = Vy
    OrVxVy(u8, u8),      // 8XY1: Vx |= Vy
    AndVxVy(u8, u8),     // 8XY2: Vx &= Vy
    XorVxVy(u8, u8),     // 8XY3: Vx ^= Vy
    AddVxVy(u8, u8),     // 8XY4: Vx += Vy, VF=进位
    SubVxVy(u8, u8),     // 8XY5: Vx -= Vy, VF=非借位
    ShrVx(u8, u8),       // 8XY6: Vx >>= 1, VF=移出位
    SubnVxVy(u8, u8),    // 8XY7: Vx = Vy - Vx, VF=非借位
    ShlVx(u8, u8),       // 8XYE: Vx <<= 1, VF=移出位
    SetI(u16),           // ANNN: I = NNN
    Draw(u8, u8, u8),    // DXYN: 在 (Vx,Vy) 绘制 N 行精灵
    LdDTVx(u8),          // FX07: Vx = DT
    LdKeyVx(u8),         // FX0A: 等待按键, 存入 Vx
    SetDTVx(u8),         // FX15: DT = Vx
    SetSTVx(u8),         // FX18: ST = Vx
    AddIVx(u8),          // FX1E: I += Vx
    LdSpriteI(u8),       // FX29: I = 字符 Vx 的字模地址
    BcdVx(u8),           // FX33: Vx 的 BCD 存入 I..I+2
    StoreRegs(u8),       // FX55: V0~Vx → [I]
    LoadRegs(u8),        // FX65: [I] → V0~Vx
    SkipNeqVxVy(u8, u8), // 9XY0: Vx != Vy 时跳过
    JumpV0(u16),         // BNNN: PC = NNN + V0
    Rand(u8, u8),        // CXNN: Vx = rand() & NN
    SkipKey(u8),         // EX9E: 按下 Vx 对应键时跳过
    SkipNotKey(u8),      // EXA1: 未按 Vx 对应键时跳过
    Unknown(u16),        // 未识别指令
}

impl Instruction {
    /// 将 16 位操作码译码为指令
    pub fn decode(opcode: u16) -> Self {
        let n1 = (opcode & 0xF000) >> 12;
        let n2 = (opcode & 0x0F00) >> 8;
        let n3 = (opcode & 0x00F0) >> 4;
        let n4 = opcode & 0x000F;

        let x = n2 as u8;
        let y = n3 as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;
        let n = n4 as u8;

        match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => Instruction::ClearScreen,
            (0x0, 0x0, 0xE, 0xE) => Instruction::Return,
            (0x1, _, _, _) => Instruction::Jump(nnn),
            (0x2, _, _, _) => Instruction::Call(nnn),
            (0x3, _, _, _) => Instruction::SkipEqVxN(x, nn),
            (0x4, _, _, _) => Instruction::SkipNeqVxN(x, nn),
            (0x5, _, _, 0x0) => Instruction::SkipEqVxVy(x, y),
            (0x6, _, _, _) => Instruction::SetVx(x, nn),
            (0x7, _, _, _) => Instruction::AddVx(x, nn),
            (0x8, _, _, 0x0) => Instruction::SetVxVy(x, y),
            (0x8, _, _, 0x1) => Instruction::OrVxVy(x, y),
            (0x8, _, _, 0x2) => Instruction::AndVxVy(x, y),
            (0x8, _, _, 0x3) => Instruction::XorVxVy(x, y),
            (0x8, _, _, 0x4) => Instruction::AddVxVy(x, y),
            (0x8, _, _, 0x5) => Instruction::SubVxVy(x, y),
            (0x8, _, _, 0x6) => Instruction::ShrVx(x, y),
            (0x8, _, _, 0x7) => Instruction::SubnVxVy(x, y),
            (0x8, _, _, 0xE) => Instruction::ShlVx(x, y),
            (0xA, _, _, _) => Instruction::SetI(nnn),
            (0xD, _, _, _) => Instruction::Draw(x, y, n),
            (0xF, _, 0x0, 0x7) => Instruction::LdDTVx(x),
            (0xF, _, 0x0, 0xA) => Instruction::LdKeyVx(x),
            (0xF, _, 0x1, 0x5) => Instruction::SetDTVx(x),
            (0xF, _, 0x1, 0x8) => Instruction::SetSTVx(x),
            (0xF, _, 0x1, 0xE) => Instruction::AddIVx(x),
            (0xF, _, 0x2, 0x9) => Instruction::LdSpriteI(x),
            (0xF, _, 0x3, 0x3) => Instruction::BcdVx(x),
            (0xF, _, 0x5, 0x5) => Instruction::StoreRegs(x),
            (0xF, _, 0x6, 0x5) => Instruction::LoadRegs(x),
            (0x9, _, _, 0x0) => Instruction::SkipNeqVxVy(x, y),
            (0xB, _, _, _) => Instruction::JumpV0(nnn),
            (0xC, _, _, _) => Instruction::Rand(x, nn),
            (0xE, _, 0x9, 0xE) => Instruction::SkipKey(x),
            (0xE, _, 0xA, 0x1) => Instruction::SkipNotKey(x),
            _ => Instruction::Unknown(opcode),
        }
    }
}
