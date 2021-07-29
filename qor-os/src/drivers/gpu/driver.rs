use crate::*;

const TEXT_MODE_WIDTH: usize = 70;
const TEXT_MODE_HEIGHT: usize = 30;

/// Text Mode Cell
#[derive(Debug, Clone, Copy)]
pub struct TextModeCell
{
    c: u8,
    fg: u8,
    bg: u8
}

/// Text Mode Data
#[derive(Debug, Clone, Copy)]
pub struct TextModeData
{
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
            buffer: [TextModeCell { c: ' ' as u8, fg, bg}; TEXT_MODE_WIDTH * TEXT_MODE_HEIGHT],
            cursor_pos: (0, 0),
            bg, fg
        }
    }
}

/// Graphics Mode
#[derive(Debug, Clone, Copy)]
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
        let (width, height) = self.driver.get_size();
        self.driver.invalidate(0, 0, width, height);
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

    /// Write a character to a position on screen
    pub fn write_character(&mut self, c: u8)
    {
        if let GraphicsMode::PseudoTextMode(data) = &mut self.mode
        {
            if c == '\n' as u8
            {
                data.cursor_pos.0 = 0;
                data.cursor_pos.1 += 1;
            }
            else
            {
                let (x, y) = data.cursor_pos;

                data.buffer[x + y * TEXT_MODE_WIDTH] = TextModeCell { c: c as u8, fg: data.fg, bg: data.bg };

                data.cursor_pos.0 += 1;

                if data.cursor_pos.0 >= TEXT_MODE_WIDTH
                {
                    data.cursor_pos.0 = 0;
                    data.cursor_pos.1 += 1;
                }

                self.update_character(x, y);
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
}