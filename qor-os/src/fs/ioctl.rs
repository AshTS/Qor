use crate::*;

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
}