use crate::*;

const TEXT_MODE_WIDTH: usize = 70;
const TEXT_MODE_HEIGHT: usize = 30;

use crate::drivers::generic::*;
use crate::fs::ioctl::IOControlCommand;

use super::structs::*;

#[derive(Debug, Clone)]
pub enum ParserState
{
    WaitingForEscape(usize),
    WaitingForArgs(Vec<usize>),
}

/// ANSI Escape Sequence Parser
#[derive(Debug, Clone)]
pub struct ANSIParser
{
    state: ParserState,
    command: Option<(Vec<usize>, char)>
}

impl ANSIParser
{
    pub fn new() -> Self
    {
        Self
        {
            state: ParserState::WaitingForEscape(0),
            command: None
        }
    }

    pub fn get_command(&mut self) -> Option<(Vec<usize>, char)>
    {
        self.command.take()
    }

    pub fn report_char(&mut self, c: u8) -> bool
    {
        let mut consumed = true;
        self.state = match &self.state
        {
            ParserState::WaitingForEscape(i) => 
            {
                match (i, c)
                {
                    (0, 0x1B) =>
                    {
                        ParserState::WaitingForEscape(1)
                    },
                    (1, 0x5B) =>
                    {
                        ParserState::WaitingForArgs(vec![0])
                    },
                    _ =>
                    {
                        consumed = false;
                        ParserState::WaitingForEscape(0)
                    }
                }
            },
            ParserState::WaitingForArgs(args) => 
            {
                match c as char
                {
                    ';' =>
                    {
                        let mut next = args.clone();
                        next.push(0);

                        ParserState::WaitingForArgs(next)
                    },
                    '0'..='9' =>
                    {
                        let v = c - 0x30;

                        let mut next = args.clone();
                        let l = next.len() - 1;
                        next[l] *= 10;
                        next[l] += v as usize;

                        ParserState::WaitingForArgs(next)
                    },
                    'm' | 'H' =>
                    {
                        let mut args = args.clone();
                        if args.len() == 0
                        {
                            args.push(0);

                        }
                        self.command = Some((args, c as char));

                        ParserState::WaitingForEscape(0)
                    },
                    _ => 
                    {
                        consumed = false;
                        ParserState::WaitingForEscape(0)
                    }
                }
            },
        };

        consumed
    }
}

pub fn ansi_to_ega(c: u8) -> u8
{
    match c
    {
        1 => 4,
        3 => 6,
        4 => 1,
        6 => 3,
        _ => 
        {
            8 + ansi_to_ega(c & 7)
        }
    }
}

/// Text Mode Cell
#[derive(Debug, Clone, Copy)]
pub struct TextModeCell
{
    c: u8,
    fg: u8,
    bg: u8
}

/// Text Mode Data
#[derive(Debug, Clone)]
pub struct TextModeData
{
    parser: ANSIParser,
    buffer: [TextModeCell; TEXT_MODE_WIDTH * TEXT_MODE_HEIGHT],
    cursor_pos: (usize, usize),
    bg: u8,
    fg: u8
}

impl core::default::Default for TextModeData
{
    fn default() -> Self
    {
        let fg = 15;
        let bg = 0;

        Self
        {
            parser: ANSIParser::new(),
            buffer: [TextModeCell { c: ' ' as u8, fg, bg}; TEXT_MODE_WIDTH * TEXT_MODE_HEIGHT],
            cursor_pos: (0, 0),
            bg, fg
        }
    }
}

impl TextModeData
{
    /// Add a newline
    pub fn newline(&mut self)
    {
        self.cursor_pos.0 = 0;
        self.cursor_pos.1 += 1;

        if self.cursor_pos.1 >= TEXT_MODE_HEIGHT
        {
            let amt = 1 + self.cursor_pos.1 - TEXT_MODE_HEIGHT;
            self.scroll(amt);
        }
    }

    /// Scroll by some amount
    pub fn scroll(&mut self, mut amt: usize)
    {
        if amt <= TEXT_MODE_HEIGHT
        {
            for y in 0..TEXT_MODE_HEIGHT - amt
            {
                for x in 0..TEXT_MODE_WIDTH
                {
                    self.buffer[x + y * TEXT_MODE_WIDTH] = self.buffer[x + (y + amt) * TEXT_MODE_WIDTH];
                }
            }

            if self.cursor_pos.1 <= amt
            {
                self.cursor_pos.1 = 0;
            }
            else
            {
                self.cursor_pos.1 -= amt;
            }
        }

        if amt > TEXT_MODE_HEIGHT
        {
            amt = TEXT_MODE_HEIGHT;
        }

        
        for y in (TEXT_MODE_HEIGHT - amt)..=TEXT_MODE_HEIGHT - 1
        {
            for x in 0..TEXT_MODE_WIDTH
            {
                self.buffer[x + y * TEXT_MODE_WIDTH] = TextModeCell { c: 0x20, fg: self.fg, bg: self.bg };
            }
        }
    }
}

/// Graphics Mode
#[derive(Debug, Clone)]
pub enum GraphicsMode
{
    PseudoTextMode(TextModeData),
    Off
}

/// Generic Graphics Driver
pub struct GenericGraphics
{
    driver: &'static mut crate::drivers::virtio::drivers::gpu::GPUDriver,
    mode: GraphicsMode
}

impl GenericGraphics
{
    /// Create a new generic graphics driver
    pub fn new(driver: &'static mut crate::drivers::virtio::drivers::gpu::GPUDriver) -> Self
    {
        Self
        {
            driver,
            mode: GraphicsMode::PseudoTextMode(TextModeData::default())
        }
    }

    /// Initialize the driver
    pub fn init(&mut self)
    {
        self.driver.init();
        self.force_update();
    }

    /// Force a screen update
    pub fn force_update(&mut self)
    {
        if let GraphicsMode::PseudoTextMode(_) = self.mode
        {
            self.invalidate_screen();
        }
        else
        {
            let (width, height) = self.driver.get_size();
            self.driver.invalidate(0, 0, width, height);
        }
    }
    
    /// Update a character location
    fn update_character(&mut self, x: usize, y: usize)
    {
        if let GraphicsMode::PseudoTextMode(data) = &mut self.mode
        {
            let cell = data.buffer[x + y * TEXT_MODE_WIDTH];

            let c = cell.c;

            let fg = crate::resources::colors::ega::EGA_COLORS[cell.fg as usize];
            let bg = crate::resources::colors::ega::EGA_COLORS[cell.bg as usize];

            self.driver.write_glpyh(&crate::resources::fonts::vga::GLYPHS[c as usize], x, y, fg, bg);
        }
    }

    /// Invalidate a character location
    fn invalidate_character(&mut self, x: usize, y: usize)
    {
        if let GraphicsMode::PseudoTextMode(_) = self.mode
        {
            self.update_character(x, y);
            self.driver.invalidate(x * 9, y * 16, 9, 16);
        }
    }

    /// Invalidate a line
    fn invalidate_line(&mut self, y: usize)
    {
        if let GraphicsMode::PseudoTextMode(_) = self.mode
        {
            for x in 0..TEXT_MODE_WIDTH
            {
                self.update_character(x, y);
            }

            self.driver.invalidate(0, y * 16, TEXT_MODE_WIDTH * 9, 16);
        }
    }

    /// Invalidate the entire screen
    fn invalidate_screen(&mut self)
    {
        for y in 0..TEXT_MODE_HEIGHT
        {
            for x in 0..TEXT_MODE_WIDTH
            {
                self.update_character(x, y);
            }
        }

        self.driver.invalidate(0, 0, TEXT_MODE_WIDTH * 9, TEXT_MODE_HEIGHT * 16);
    }

    /// Write a character to a position on screen
    pub fn write_character(&mut self, c: u8)
    {
        if let GraphicsMode::PseudoTextMode(data) = &mut self.mode
        {
            if data.parser.report_char(c)
            {
                if let Some(cmd) = data.parser.get_command()
                {
                    if cmd.1 == 'm' && cmd.0[0] == 0
                    {
                        data.fg = 15;
                        data.bg = 0;
                    }
                    else if cmd.1 == 'm' && cmd.0[0] >= 30 && cmd.0[0] <= 37
                    {
                        data.fg = ansi_to_ega(cmd.0[0] as u8 - 30);
                    }
                    else if cmd.1 == 'm' && cmd.0[0] >= 90 && cmd.0[0] <= 97
                    {
                        data.fg = ansi_to_ega(8 + cmd.0[0] as u8 - 90);
                    }
                    else if cmd.1 == 'm' && cmd.0[0] >= 40 && cmd.0[0] <= 47
                    {
                        data.bg = ansi_to_ega(cmd.0[0] as u8 - 40);
                    }
                    else if cmd.1 == 'm' && cmd.0[0] >= 100 && cmd.0[0] <= 107
                    {
                        data.bg = ansi_to_ega(8 + cmd.0[0] as u8 - 100);
                    }
                    else if cmd.1 == 'H'
                    {
                        data.cursor_pos = (cmd.0[0].max(1) - 1, cmd.0[1].max(1) - 1);
                    }
                    else
                    {
                        kwarnln!("Cmd: {:?}", cmd);
                    }
                }

                return;
            }

            if c == 10 || c == 13
            {
                data.newline();
            }
            else if c == 8 || c == 127
            {
                data.cursor_pos.0 = data.cursor_pos.0.max(1) - 1;

                let (x, y) = data.cursor_pos;

                data.buffer[x + y * TEXT_MODE_WIDTH] = TextModeCell { c: ' ' as u8, fg: data.fg, bg: data.bg };
            }
            else
            {
                let (x, y) = data.cursor_pos;

                data.buffer[x + y * TEXT_MODE_WIDTH] = TextModeCell { c: c as u8, fg: data.fg, bg: data.bg };

                data.cursor_pos.0 += 1;

                if data.cursor_pos.0 >= TEXT_MODE_WIDTH
                {
                    data.newline();
                }
            }
        }
    }

    /// Write a string to the screen
    pub fn write_string(&mut self, s: &str)
    {
        for c in s.chars()
        {
            self.write_character(c as u8);
        }
    }

    /// Execute an ioctl command for this driver
    pub fn exec_ioctl(&mut self, ioctl: IOControlCommand) -> usize
    {
        match ioctl
        {
            IOControlCommand::FrameBufferGetFixedInfo { response } => 
            {
                let (width, height) = self.driver.get_size();

                *response = FramebufferFixedInfo
                    {
                        name: ['V' as u8, 'i' as u8, 'r' as u8, 't' as u8,
                               'I' as u8, 'O' as u8, ' ' as u8, 'G' as u8,
                               'P' as u8, 'U' as u8, 0, 0, 0, 0, 0, 0],
                        buffer_start: self.driver.frame_buffer.data as u64,
                    
                        buffer_len: (width * height * 4) as u32,
                        fb_type: 0x1000,
                        aux_type: 0,
                        visual: 2,
                        x_pan_step: 0,
                        y_pan_step: 0,
                        y_wrap_step: 0,
                    
                        line_length: (width * 4) as u32,
                    
                        mmio_len: 0,
                        accel: 0,
                    
                        capabilities: 0,
                        reserved: [0; 2]
                    };

                0
            },
            IOControlCommand::FrameBufferPutVariableInfo { .. } => 
            {
                // TODO: Add variability here

                // Nothing here can be edited so far, so we will have to leave
                // this returning -1 until such features can be implemented
                usize::MAX
            },
            IOControlCommand::FrameBufferGetVariableInfo { response } => 
            {
                let (width, height) = self.driver.get_size();

                *response = FramebufferVariableInfo
                    {
                        x_res: width as u32,
                        y_res: height as u32,

                        x_res_virtual: width as u32,
                        y_res_virtual: height as u32,
                        x_offset: 0,
                        y_offset: 0,

                        bits_per_pixel: 32,
                        grayscale: 0,

                        red: FramebufferBitfield { offset: 0, length: 8, msb_right: 0 },
                        green: FramebufferBitfield { offset: 8, length: 8, msb_right: 0 },
                        blue: FramebufferBitfield { offset: 16, length: 8, msb_right: 0 },
                        transp: FramebufferBitfield { offset: 24, length: 8, msb_right: 0 },

                        non_std: 0,

                        activate: 0,

                        height: height as u32 / 80,
                        width: width as u32 / 80,

                        obsolete_flags: 0,

                        unused_timing: [0; 15]
                    };
                0
            },
        }
    }
}

impl ByteInterface for GenericGraphics
{
    fn read_byte(&mut self) -> Option<u8>
    {
        None
    }

    fn write_byte(&mut self, data: u8)
    {
        self.write_character(data)
    }

    fn flush(&mut self)
    {
        self.invalidate_screen();
    }
}

impl BufferInterface for GenericGraphics
{
    fn read_byte(&mut self, offset: usize) -> Option<u8>
    {
        if offset < self.get_size()
        {
            Some(unsafe { (self.driver.frame_buffer.data as *mut u8).add(offset).read() })
        }
        else
        {
            None
        }
    }

    fn write_byte(&mut self, offset: usize, data: u8)
    {
        if offset < self.get_size()
        {
            unsafe { (self.driver.frame_buffer.data as *mut u8).add(offset).write(data) }
        }
    }

    fn get_size(&self) -> usize
    {
        let (w, h) = self.driver.frame_buffer.get_size();

        4 * w * h
    }

    fn flush(&mut self)
    {
        let (w, h) = self.driver.frame_buffer.get_size();
        self.driver.invalidate(0, 0, w, h)
    }
}