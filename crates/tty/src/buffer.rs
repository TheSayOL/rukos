//! Here document functions for taking care of tty buffer and their flipping.
//! Drivers are supposed to fill the buffer by one of those functions below
//! and then flip the buffer, so that the data are passed to line discipline for further processing.

const TTY_BUF_SIZE: usize = 4096;

/// ring buffer.
#[derive(Debug)]
struct RingBuffer {
    /// data.
    buf: [u8; TTY_BUF_SIZE],
    /// the first element or empty slot if buffer is empty.
    head: usize,
    /// the first empty slot.
    tail: usize,
    /// number of elements.
    len: usize,
}

/// characters buffer.
#[derive(Debug)]
pub struct TtyBuffer {
    buffer: spinlock::SpinNoIrq<RingBuffer>,
}

impl TtyBuffer {
    pub fn new() -> Self {
        let buf = RingBuffer {
            buf: [0u8; TTY_BUF_SIZE],
            head: 0,
            tail: 0,
            len: 0,
        };
        Self {
            buffer: spinlock::SpinNoIrq::new(buf),
        }
    }

    /// flush buffer.
    pub fn flush(&self) {
        let mut buf = self.buffer.lock();
        buf.len = 0;
        buf.head = 0;
        buf.tail = 0;
    }

    /// get buffer's index'th element.
    pub fn see(&self, index: usize) -> u8 {
        let buf = self.buffer.lock();
        if index < buf.len {
            buf.buf[(index + buf.head) % TTY_BUF_SIZE]
        } else {
            0
        }
    }

    /// push a charachter
    pub fn push(&self, ch: u8) {
        let mut buf = self.buffer.lock();
        if buf.len != TTY_BUF_SIZE {
            buf.len += 1;
            let idx = buf.tail;
            buf.buf[idx] = ch;
            buf.tail = (buf.tail + 1) % TTY_BUF_SIZE;
        }
    }

    /// delete and return last character.
    pub fn pop(&self) -> u8 {
        self.delete(0)
        // let mut buf = self.buffer.lock();
        // if buf.len != 0 {
        //     buf.len -= 1;
        //     buf.tail = (buf.tail - 1) % TTY_BUF_SIZE;
        //     buf.buf[buf.tail]
        // } else {
        //     0
        // }
    }

    /// insert char `ch` to buffer's index'th slot.
    pub fn insert(&self, ch: u8, index: usize) {
        let mut buf = self.buffer.lock();
        // if not full and index is right
        if buf.len != TTY_BUF_SIZE && index <= buf.len {
            // shift buffer[index..move_len+index] one slot right.
            let move_len = buf.len - index;
            let mut i = buf.tail;
            for _ in 0..move_len {
                i -= 1;
                buf.buf[(i + 1) % TTY_BUF_SIZE] = buf.buf[i % TTY_BUF_SIZE];
            }
            // insert
            let idx = (buf.head + index) % TTY_BUF_SIZE;
            buf.buf[idx] = ch;
            buf.len += 1;
            buf.tail = (buf.tail + 1) % TTY_BUF_SIZE;
        }
    }

    /// delete a character in buffer[index].
    pub fn delete(&self, index: usize) -> u8 {
        let mut buf = self.buffer.lock();
        // if not empty and index is right
        if buf.len != 0 && index < buf.len {
            let move_len = buf.len - index;
            let mut i = index + buf.head;

            // save retval
            let ret = buf.buf[i % TTY_BUF_SIZE];

            // copy move_len elements from buffer[index+head] to buffer[index+head-1];
            for _ in 0..move_len {
                buf.buf[i % TTY_BUF_SIZE] = buf.buf[(i + 1) % TTY_BUF_SIZE];
                i += 1;
            }

            // len -= 1
            buf.len -= 1;
            buf.tail -= 1;
            ret
        } else {
            0
        }
    }

    /// get index
    pub fn len(&self) -> usize {
        self.buffer.lock().len
    }
}

#[derive(Debug)]
pub struct EchoBuffer {
    pub buffer: TtyBuffer,
    pub col: usize,
}

impl EchoBuffer {
    pub fn new() -> Self {
        Self {
            buffer: TtyBuffer::new(),
            col: 0,
        }
    }
    pub fn col(&self) -> usize {
        self.col
    }
    pub fn col_sub_one(&mut self) {
        self.col -= 1;
    }
    pub fn col_add_one(&mut self) {
        self.col += 1;
    }
    pub fn col_clear(&mut self) {
        self.col = 0;
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
