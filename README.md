# CHIP-8 模拟器（Rust）

用 Rust 从零实现的 CHIP-8 虚拟机：完整的取指—译码—执行核心、64×32 显示器、16 键键盘、延迟/声音定时器，以及一个可替换的外设抽象层（`Platform` trait）。

随附 `games/` 下的经典 ROM 与 `演示视频.mp4` 可直接体验。

## 特性

- **完整指令集**：实现标准 CHIP-8 全部 35 条指令，包括 `8XY6`/`8XYE` 移位、`FX0A` 等待按键、`FX33` BCD 转换等易漏项。
- **核心与平台解耦**：`cpu.rs` 只依赖 `Platform` trait（`poll_keys` / `render` / `beep`），因此可以换任意前端（Web、嵌入式、无头测试）。
- **桌面前端**：基于 `minifb` 的 640×320 窗口（64×32 像素，放大 10 倍），帧率限制在约 60 FPS。
- **声音**：`rodio` 生成的 440 Hz 正弦波，由声音定时器（ST）驱动开关。
- **健壮性**：未知操作码、栈溢出/下溢、PC 越界、ROM 过大均返回 `Result` 错误而非 panic，主循环捕获后停机并打印原因。
- **单元测试**：核心逻辑附带测试（初始状态、跳转、加法溢出、清屏、调用/返回）。

## 环境要求

- Rust 1.85 或更高（项目使用 `edition = "2024"`）
- 可用的桌面与音频环境：Windows / macOS 开箱即用；Linux 需要 ALSA 开发库（如 `libasound2-dev`）

## 快速开始

```bash
git clone https://github.com/SSDGK/CHIP-8-Rust.git
cd CHIP-8-Rust

# 运行随附 ROM（路径含空格/方括号时务必加引号）
cargo run --release -- "games/Tetris [Fran Dachille, 1991].ch8"
cargo run --release -- games/SpaceInvaders.ch8
cargo run --release -- games/test_opcode.ch8

# 运行全部单元测试
cargo test
```

`cargo run` 后面 `--` 之后的第一个参数即 ROM 文件路径，扩展名不限。

编译产物位于 `target/release/chip8`（Windows 为 `chip8.exe`），可直接用已编译的程序运行 ROM：

```bash
./target/release/chip8 "games/IBMLogo.ch8"
```

## 键位映射

CHIP-8 的 4×4 十六进制键盘映射到 PC 键盘（沿用 COSMAC VIP 的左手布局）：

| CHIP-8 键位 | | | |
|:---:|:---:|:---:|:---:|
| `1` → `1` | `2` → `2` | `3` → `3` | `C` → `4` |
| `4` → `Q` | `5` → `W` | `6` → `E` | `D` → `R` |
| `7` → `A` | `8` → `S` | `9` → `D` | `E` → `F` |
| `A` → `Z` | `0` → `X` | `B` → `C` | `F` → `V` |

- `Esc` 退出程序，关闭窗口同样退出。

## 已实现的指令

| 操作码 | 说明 | 操作码 | 说明 |
|---|---|---|---|
| `00E0` | 清屏 | `9XY0` | `VX != VY` 则跳过下一条 |
| `00EE` | 从子程序返回 | `ANNN` | `I = NNN` |
| `1NNN` | 跳转到 `NNN` | `BNNN` | `PC = NNN + V0` |
| `2NNN` | 调用子程序 `NNN` | `CXNN` | `VX = rand() & NN` |
| `3XNN` | `VX == NN` 则跳过 | `DXYN` | 绘制精灵，`VF` = 碰撞 |
| `4XNN` | `VX != NN` 则跳过 | `EX9E` | 按下 `VX` 对应的键则跳过 |
| `5XY0` | `VX == VY` 则跳过 | `EXA1` | 未按下 `VX` 对应的键则跳过 |
| `6XNN` | `VX = NN` | `FX07` | `VX = DT` |
| `7XNN` | `VX += NN` | `FX0A` | 等待按键，结果存入 `VX` |
| `8XY0` | `VX = VY` | `FX15` | `DT = VX` |
| `8XY1` | `VX \|= VY` | `FX18` | `ST = VX` |
| `8XY2` | `VX &= VY` | `FX1E` | `I += VX` |
| `8XY3` | `VX ^= VY` | `FX29` | `I = 字符 VX 的字模地址` |
| `8XY4` | `VX += VY`，`VF` = 进位 | `FX33` | `VX` 的 BCD 存入 `I..I+2` |
| `8XY5` | `VX -= VY`，`VF` = 非借位 | `FX55` | `V0..VX` → `[I]` |
| `8XY6` | `VX >>= 1`，`VF` = 移出位 | `FX65` | `[I]` → `V0..VX` |
| `8XY7` | `VX = VY - VX`，`VF` = 非借位 | | |
| `8XYE` | `VX <<= 1`，`VF` = 移出位 | | |

## 目录结构

```text
chip8/
├── Cargo.toml           # 依赖：minifb（窗口/输入）、rodio（音频）、rand（CXNN）
├── games/               # 随附 ROM：Tetris、SpaceInvaders、IBMLogo、Airplane、BC_test、test_opcode
├── src/
│   ├── main.rs          # 主循环：加载 ROM → 每帧执行 10 条指令 → 更新定时器 → 渲染
│   ├── cpu.rs           # 虚拟机核心：内存、寄存器、栈、定时器、显示、字模、单元测试
│   ├── instruction.rs   # 16 位操作码 → Instruction 枚举的译码表
│   ├── platform.rs      # DesktopPlatform：minifb 窗口、键盘映射、像素渲染
│   └── audio.rs         # Beeper：rodio 正弦波蜂鸣器
└── 演示视频.mp4
```

## 工作原理

- **内存布局**：共 4 KB。`0x000–0x04F` 保留，`0x050–0x09F` 存放内置 4×5 字模（0–F 共 16 个字符，每字符 5 字节），`0x200` 起加载 ROM，因此单个 ROM 上限为 3584 字节。
- **执行模型**：`Chip8::tick()` 先轮询键盘，再取 16 位操作码，经 `Instruction::decode` 译码后由 `execute` 执行。主循环每帧执行 10 条指令，窗口刷新被限制在约 60 FPS，因此大约 600 指令/秒。
- **定时器**：`update_timers()` 每帧调用一次，`DT` 与 `ST` 各减 1（约 60 Hz）；`ST > 0` 时蜂鸣器开启。
- **精灵绘制**：`DXYN` 以 XOR 方式写入显示缓冲区，若有像素由亮变灭则将 `VF` 置 1（碰撞检测）。
- **错误处理**：致命错误以 `Result<(), String>` 向上返回，`main` 打印 `CPU 停机: <原因>` 后停止执行指令，但窗口仍保持响应用于排错。

## 已知限制

- `DXYN` 在精灵越过屏幕右/下边界时直接截断，未实现部分解释器的环绕（wrap）行为。
- `FX0A` 采用「无键按下则重试同一条指令」的实现，不等待按键释放，个别 ROM 可能表现不同。
- `8XY6` / `8XYE` 使用经典行为（对 `VX` 自身移位，忽略 `VY`）。
- 仅实现基础 CHIP-8，未实现 SCHIP/SUPER-CHIP 扩展（高分辨率模式、`00CN`、`FX30`、`FX75`、`FX85` 等）。

## 参考资料

- Tobias V. Langhoff, *Guide to making a CHIP-8 emulator*
- Cowgod's CHIP-8 Technical Reference
- `games/test_opcode.ch8` 与 `games/BC_test.ch8` 可用于验证指令实现是否正确

## 许可证

本仓库当前未包含 LICENSE 文件。如需公开发布，建议补充许可证（例如 MIT 或 Apache-2.0）。