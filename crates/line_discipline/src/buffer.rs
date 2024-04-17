//! Here document functions for taking care of tty buffer and their flipping.
//! Drivers are supposed to fill the buffer by one of those functions below
//! and then flip the buffer, so that the data are passed to line discipline for further processing.

use spin::mutex::Mutex;

const TTY_BUF_SIZE: usize = 4096;

/// a buffer to save characters.
#[derive(Debug)]
pub struct TtyBuffer {
    inner: Mutex<TtyBufferInner>,
}

#[derive(Debug)]
struct TtyBufferInner {
    buf: [u8; TTY_BUF_SIZE],
    len: usize,
}

impl TtyBuffer {
    pub fn new() -> Self {
        let inner = TtyBufferInner {
            buf: [0u8; TTY_BUF_SIZE],
            len: 0,
        };
        let inner = Mutex::new(inner);
        Self { inner }
    }
    /// flush buffer.
    pub fn flush(&self) {
        self.inner.lock().flush()
    }

    /// get buffer[index] without modifying buffer.
    pub fn see(&self, index: usize) -> u8 {
        self.inner.lock().see(index)
    }

    pub fn full(&self) -> bool {
        self.inner.lock().full()
    }

    pub fn empty(&self) -> bool {
        self.inner.lock().empty()
    }

    /// insert a character `ch` to buffer[index].
    pub fn insert(&self, ch: u8, index: usize) {
        self.inner.lock().insert(ch, index)
    }

    /// delete last character in buffer.
    pub fn pop(&self) {
        self.inner.lock().pop()
    }

    /// push a charachter to buffer
    pub fn push(&self, ch: u8) {
        self.inner.lock().push(ch)
    }

    /// push characters to buffer
    pub fn push_many(&self, chars: &[u8]) {
        self.inner.lock().push_many(chars)
    }

    /// delete a character in buffer[index].
    pub fn delete(&self, index: usize) -> u8 {
        self.inner.lock().delete(index)
    }

    /// get index
    pub fn len(&self) -> usize {
        self.inner.lock().len
    }
}

impl TtyBufferInner {
    /// flush buffer.
    pub fn flush(&mut self) {
        self.len = 0;
    }

    pub fn see(&self, index: usize) -> u8 {
        if index < TTY_BUF_SIZE {
            self.buf[index]
        } else {
            0
        }
    }

    pub fn full(&self) -> bool {
        self.len == TTY_BUF_SIZE
    }

    pub fn empty(&self) -> bool {
        self.len == 0
    }

    /// insert a character `ch` to buffer[index].
    pub fn insert(&mut self, ch: u8, index: usize) {
        if !self.full() && index < self.len {
            // copy buffer[index..len] to buffer[index+1..len+1]
            for i in (index..self.len).rev() {
                self.buf[i + 1] = self.buf[i];
            }
            // insert
            self.buf[index] = ch;
            self.len += 1;
        }
    }

    /// delete last character in buffer.
    pub fn pop(&mut self) {
        if !self.empty() {
            self.len -= 1;
        }
    }

    /// push a charachter to buffer
    pub fn push(&mut self, ch: u8) {
        if !self.full() {
            self.buf[self.len] = ch;
            self.len += 1;
        }
    }

    /// push characters to buffer
    pub fn push_many(&mut self, chars: &[u8]) {
        // only copy when not overflow
        if self.len + chars.len() <= TTY_BUF_SIZE {
            for ch in chars {
                self.push(*ch);
            }
            self.len += chars.len();
        }
    }

    /// delete a character in buffer[index].
    pub fn delete(&mut self, index: usize) -> u8 {
        let mut ret = 0;
        if !self.empty() && index < self.len {
            // save retval
            ret = self.buf[index];

            // copy buffer[index+1..len] to buffer[index..len-1];
            for i in index + 1..self.len {
                self.buf[i - 1] = self.buf[i];
            }

            // len -= 1
            self.len -= 1;
        }
        ret
    }

    /// get buf.
    pub fn buf(&self) -> &[u8] {
        &self.buf
    }

    /// get index
    pub fn len(&self) -> usize {
        self.len
    }
}

// /// Queue a series of bytes to the tty buffering.
// /// All the characters passed are marked with the supplied flag.
// /// Return: the number added.
// pub fn tty_insert_flip_string_fixed_flag(
//     port: *mut TtyPort,
//     chars: *const c_uchar,
//     flag: c_char,
//     size: c_size_t,
// ) -> c_int {
//     unimplemented!()
// }

// /// Queue a series of bytes to the tty buffering.
// /// For each character the flags array indicates the status of the character.
// /// Return: the number added.
// pub fn tty_insert_flip_string_flags(
//     port: *mut TtyPort,
//     chars: *const c_uchar,
//     flags: *const c_char,
//     size: c_size_t,
// ) -> c_int {
//     unimplemented!()
// }

// /// Queue a single byte `ch` to the tty buffering, with an optional flag.
// /// This is the slow path of tty_insert_flip_char().
// pub fn __tty_insert_flip_char(port: *mut TtyPort, ch: c_uchar, flag: c_char) -> c_int {
//     unimplemented!()
// }

// /// Prepare a block of space in the buffer for data.
// /// This is used for drivers that need their own block copy routines into the buffer.
// /// There is no guarantee the buffer is a DMA target!
// /// `chars`: return pointer for character write area
// /// Return: the length available and `chars` to the space which is now allocated
// /// and accounted for as ready for normal characters.
// pub fn tty_prepare_flip_string(
//     port: *mut TtyPort,
//     chars: *mut *mut c_uchar,
//     size: c_size_t,
// ) -> c_int {
//     unimplemented!()
// }

// /// forward data to line discipline
// /// Callers other than flush_to_ldisc() need to exclude the kworker
// /// from concurrent use of the line discipline, see paste_selection().
// /// `p`: char buffer
// /// `f`: TTY_NORMAL, TTY_BREAK, etc. flags buffer
// /// Return: the number of bytes processed.
// pub fn tty_ldisc_receive_buf(
//     ld: *mut TtyLdisc,
//     p: *const c_uchar,
//     f: *const c_char,
//     count: c_int,
// ) -> c_int {
//     unimplemented!()
// }

// /// Queue a push of the terminal flip buffers to the line discipline.
// /// Can be called from IRQ/atomic context.
// /// In the event of the queue being busy for flipping the work will be held off and retried later.
// pub fn tty_flip_buffer_push(port: *mut TtyPort) {
//     unimplemented!()
// }

// /// Return unused buffer space
// /// `port`: tty port owning the flip buffer
// /// Return: the # of bytes which can be written by the driver without reaching the buffer limit.
// /// Note: this does not guarantee that memory is available to write the returned # of bytes
// /// (use tty_prepare_flip_string() to pre-allocate if memory guarantee is required).
// pub fn tty_buffer_space_avail(port: *mut TtyPort) -> c_uint {
//     unimplemented!()
// }

// /// Change the tty buffer memory limit.
// /// Must be called before the other tty buffer functions are used.
// pub fn tty_buffer_set_limit(port: *mut TtyPort, limit: c_int) -> c_int {
//     unimplemented!()
// }

// /// gain exclusive access to buffer
// /// used only in special circumstances. Avoid it.
// /// Guarantees safe use of the tty_ldisc_ops.receive_buf() method
// /// by excluding the buffer work and any pending flush from using the flip buffer.
// /// Data can continue to be added concurrently to the flip buffer from the driver side.
// pub fn tty_buffer_lock_exclusive(port: *mut TtyPort) {
//     unimplemented!()
// }

// /// release exclusive access
// /// used only in special circumstances. Avoid it.
// /// The buffer work is restarted if there is data in the flip buffer.
// pub fn tty_buffer_unlock_exclusive(port: *mut TtyPort) {
//     unimplemented!()
// }

// /// free buffers used by a tty.
// /// Remove all the buffers pending on a tty whether queued with data or in the free ring.
// /// Must be called when the tty is no longer in use.
// fn tty_buffer_free_all(port: *mut TtyPort) {
//     unimplemented!()
// }

// /// Allocate a new tty buffer to hold the desired number of characters.
// /// We round our buffers off in 256 character chunks to get better allocation behaviour.
// /// `size`: desired size (characters)
// /// Return: NULL if OOM or the allocation would exceed the per device queue.
// fn tty_buffer_alloc(port: *mut TtyPort, size: c_size_t) -> *mut TtyBuffer {
//     unimplemented!()
// }

// /// Free a tty buffer, or add it to the free list according to our internal strategy.
// fn tty_buffer_free(port: *mut TtyPort, b: *mut TtyBuffer) {
//     unimplemented!()
// }

// /// Flush all the buffers containing receive data.
// /// If ld != NULL, flush the ldisc input buffer.
// /// Locking: takes buffer lock to ensure single-threaded flip buffer ‘consumer’.
// fn tty_buffer_flush(tty: *mut TtyStruct, ld: TtyLdisc) {
//     unimplemented!()
// }

// /// grow tty buffer if needed.
// /// Make at least `size` bytes of linear space available for the tty buffer.
// /// Will change over to a new buffer if the current buffer is encoded as TTY_NORMAL
// /// (so has no flags buffer) and the new buffer requires a flags buffer.
// /// `flags`: buffer flags if new buffer allocated (default = 0)
// /// Return: the size we managed to find.
// fn __tty_buffer_request_room(port: *mut TtyPort, size: c_size_t, flags: c_int) -> c_int {
//     unimplemented!()
// }

// /// Called out of the software interrupt to flush data from the buffer chain to the line discipline.
// /// The receive_buf() method is single threaded for each tty instance.
// /// `work`: tty structure passed from work queue.
// /// Locking: takes buffer lock to ensure single-threaded flip buffer ‘consumer’.
// fn flush_to_ldisc(work: *mut WorkStruct) {
//     unimplemented!()
// }

// /// Set up the initial state of the buffer management for a tty device.
// /// Must be called before the other tty buffer functions are used.
// fn tty_buffer_init(port: *mut TtyPort) {
//     unimplemented!()
// }
