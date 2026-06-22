use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};

/// CHIP-8 蜂鸣器
pub struct Beeper {
    /// 持有输出流所有权，防止被回收
    _stream: OutputStream,
    sink: Sink,
}

impl Beeper {
    /// 初始化音频系统，默认静音
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let source = SineWave::new(440.0).amplify(0.2);
        sink.append(source);
        sink.pause();

        Self { _stream, sink }
    }

    /// 控制蜂鸣器开关
    pub fn set_playing(&self, play: bool) {
        if play {
            if self.sink.is_paused() {
                self.sink.play();
            }
        } else {
            if !self.sink.is_paused() {
                self.sink.pause();
            }
        }
    }
}
