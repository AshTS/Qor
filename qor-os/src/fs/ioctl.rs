use crate::*;

pub enum IOControlCommand
{
    FrameBufferGetFixedInfo{response: &'static mut drivers::gpu::structs::FramebufferFixedInfo},
    FrameBufferPutVariableInfo{response: &'static mut drivers::gpu::structs::FramebufferVariableInfo},
    FrameBufferGetVariableInfo{response: &'static mut drivers::gpu::structs::FramebufferVariableInfo},
}