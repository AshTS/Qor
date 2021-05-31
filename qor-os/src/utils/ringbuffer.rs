// Byte Buffer Size
const BUFFER_SIZE: usize = 1024;

/// Generic Byte Based Ring Buffer
pub struct ByteRingBuffer
{
    data: [u8; BUFFER_SIZE],
    start: usize,
    end: usize
}

impl ByteRingBuffer
{
    /// Create a new, empty byte ring buffer
    pub const fn new() -> Self
    {
        Self
        {
            data: [0u8; BUFFER_SIZE],
            start: 0,
            end: 0
        }
    }

    /// Add a byte to the buffer
    pub fn enqueue_byte(&mut self, byte: u8) -> bool
    {
        if (self.end + 1) % BUFFER_SIZE == self.start
        {
            return false;
        }

        self.data[self.end] = byte;
        self.end = (self.end + 1) % BUFFER_SIZE;

        true
    }

    /// Remove a byte from the buffer
    pub fn dequeue_byte(&mut self) -> Option<u8>
    {
        if self.start == self.end
        {
            None
        }
        else
        {
            let data = Some(self.data[self.start]);

            self.start = (self.start + 1) % BUFFER_SIZE;

            data
        }
    }

    /// Remove the data at the front of the buffer
    pub fn pop_byte(&mut self) -> Option<u8>
    {
        if self.start == self.end
        {
            None
        }
        else
        {
            self.end = (self.end + BUFFER_SIZE - 1) % BUFFER_SIZE;

            Some(self.data[self.end])
        }
    }
}

/// Ring Buffer Test
#[test_case]
fn ring_buffer()
{
    let mut buffer = ByteRingBuffer::new();

    // Ensure a fresh buffer contains no data
    assert_eq!(buffer.dequeue_byte(), None);
    assert_eq!(buffer.dequeue_byte(), None);
    assert_eq!(buffer.pop_byte(), None);
    assert_eq!(buffer.pop_byte(), None);

    // Ensure pop_byte resets the state
    assert!(buffer.enqueue_byte(0x42));
    assert_eq!(buffer.pop_byte(), Some(0x42));

    assert_eq!(buffer.dequeue_byte(), None);
    assert_eq!(buffer.dequeue_byte(), None);
    assert_eq!(buffer.pop_byte(), None);
    assert_eq!(buffer.pop_byte(), None);

    // Ensure BUFFER_SIZE - 1 bytes can be written
    for i in 0..BUFFER_SIZE - 1
    {
        assert!(buffer.enqueue_byte((i & 0xFF) as u8));
    }

    // Ensure no more data can be added
    assert!(!buffer.enqueue_byte(0xFF));

    const MID_WAY: usize = if BUFFER_SIZE > 512 {256} else {BUFFER_SIZE / 2};

    // Ensure data can be removed in order and that removal adds one byte of space
    for i in 0..MID_WAY
    {
        assert_eq!(buffer.dequeue_byte(), Some((i & 0xFF) as u8));


        assert!(buffer.enqueue_byte((0xFF) as u8));
        assert!(!buffer.enqueue_byte((0xFF) as u8));
    }

    // Ensure multiple bytes can be removed at once
    for i in MID_WAY..(2 * MID_WAY - 1)
    {
        assert_eq!(buffer.dequeue_byte(), Some((i & 0xFF) as u8));
    }

    // Ensure the proper amount of space is freed
    for i in MID_WAY..(2 * MID_WAY - 1)
    {
        assert!(buffer.enqueue_byte((i & 0xFF) as u8));
        assert_eq!(buffer.pop_byte(), Some((i & 0xFF) as u8));

        assert!(buffer.enqueue_byte((i & 0xFF) as u8));
    }
}
