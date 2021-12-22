use crate::*;

use crate::process::PID;

#[derive(Debug)]
/// ioctl Commands
pub enum IOControlCommand
{
    // Framebuffer
    FrameBufferGetFixedInfo{response: &'static mut drivers::gpu::structs::FramebufferFixedInfo},
    FrameBufferPutVariableInfo{response: &'static mut drivers::gpu::structs::FramebufferVariableInfo},
    FrameBufferGetVariableInfo{response: &'static mut drivers::gpu::structs::FramebufferVariableInfo},
    FrameBufferFlush,

    // Real Time Clock
    RealTimeClockGetTime{response: &'static mut drivers::rtc::RTCTime},
    RealTimeClockGetTimestamp{response: &'static mut u64},

    // TTY
    TeletypeGetSettings{response: &'static mut fs::devfs::tty::TeletypeSettings},
    TeletypeSetSettingsNoWait{response: &'static mut fs::devfs::tty::TeletypeSettings},
    TeletypeSetSettingsDrain{response: &'static mut fs::devfs::tty::TeletypeSettings},
    TeletypeSetSettingsFlush{response: &'static mut fs::devfs::tty::TeletypeSettings},
    TeletypeGetProcessGroup{response: &'static mut PID},
    TeletypeSetProcessGroup{response: &'static mut PID},
}