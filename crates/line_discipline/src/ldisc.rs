//! TTY line discipline process all incoming and outgoing character from/to a tty device.
//! The default line discipline is N_TTY.
//! It is also a fallback if establishing any other discipline for a tty fails.
//! If even N_TTY fails, N_NULL takes over.
//! That never fails, but also does not process any characters – it throws them away.

//! Line disciplines are registered with tty_register_ldisc() passing the ldisc structure.
//! At the point of registration the discipline must be ready to use
//! and it is possible it will get used before the call returns success.
//! If the call returns an error then it won’t get called.
//! Do not re-use ldisc numbers as they are part of the userspace ABI
//! and writing over an existing ldisc will cause demons to eat your computer.
//! You must not re-register over the top of the line discipline
//! even with the same data or your computer again will be eaten by demons.
//! In order to remove a line discipline call tty_unregister_ldisc().
//! Heed this warning: the reference count field of the registered copies
//! of the tty_ldisc structure in the ldisc table counts the number of lines using this discipline.
//! The reference count of the tty_ldisc structure within a tty
//! counts the number of active users of the ldisc at this instant.
//! In effect it counts the number of threads of execution within an ldisc method
//! (plus those about to enter and exit although this detail matters not).

const LF: u8 = b'\n';
const CR: u8 = b'\r';

const DEL: u8 = b'\x7f';
const BS: u8 = b'\x08';

const SPACE: u8 = b' ';

/// starting with 27, 91
const ARROW_PREFIX: [u8; 2] = [27, 91];
const UP: u8 = 65;
const DOWN: u8 = 66;
const RIGHT: u8 = 67;
const LEFT: u8 = 68;


use alloc::sync::Arc;
use spin::Mutex;

use crate::{buffer::TtyBuffer, tty::TtyStruct};

pub enum LdiscIndex {
    RAW,
    NTty,
}

pub fn new_ldisc(index: LdiscIndex) -> Arc<TtyLdisc> {
    let ret = match index {
        _ => TtyLdisc::new(),
    };
    Arc::new(ret)
}

#[derive(Debug)]
struct EchoBuffer {
    buffer: TtyBuffer,
    col: usize,
}

impl EchoBuffer {
    fn new() -> Self {
        Self {
            buffer: TtyBuffer::new(),
            col: 0,
        }
    }
    fn col(&self) -> usize {
        self.col
    }
    fn col_sub_one(&mut self) {
        self.col -= 1;
    }
    fn col_add_one(&mut self) {
        self.col += 1;
    }
    fn col_clear(&mut self) {
        self.col = 0;
    }
    fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Debug)]
pub struct TtyLdisc {
    /// for tty_read()
    read_buf: TtyBuffer,

    /// namely echo buffer, for the data driver sent and copy to read_buf when processing done.
    echo_buf: Mutex<EchoBuffer>,
}

impl TtyLdisc {
    pub fn new() -> Self {
        Self {
            read_buf: TtyBuffer::new(),
            echo_buf: Mutex::new(EchoBuffer::new()),
        }
    }
}

impl TtyLdisc {
    pub fn name(&self) -> &str {
        "N_TTY"
    }

    pub fn num(&self) -> isize {
        LdiscIndex::NTty as _
    }

    /// kernel wants data.
    /// send to kernel if read_buf has.
    pub fn read(&self, buf: &mut [u8]) -> usize {
        // get len of read_buf
        let len = buf.len().min(self.read_buf.len());

        // copy data from read_buf to `buf`
        for i in 0..len {
            let ch = self.read_buf.delete(0);
            buf[i] = ch;
        }

        // flush
        self.read_buf.flush();

        // return len of reading
        len
    }

    /// driver sends data.
    /// handle characters and echo.
    /// seems in irq.
    pub fn receive_buf(&self, tty: Arc<TtyStruct>, buf: &[u8]) {
        let mut i = 0;
        let buf_len = buf.len();
        while i < buf_len {
            let ch = buf[i];

            // handle arrow keys
            if i + 2 < buf_len && buf[i] == ARROW_PREFIX[0] && buf[i + 1] == ARROW_PREFIX[1] {
                let ch = buf[i + 2];
                match ch {
                    LEFT => {
                        // if can left
                        let mut lock = self.echo_buf.lock();
                        if lock.col() > 0 {
                            self.write(tty.clone(), &[buf[i], buf[i + 1], ch]);
                            lock.col_sub_one();
                        }
                    }
                    RIGHT => {
                        let mut lock = self.echo_buf.lock();
                        if lock.col() < lock.len() {
                            self.write(tty.clone(), &[buf[i], buf[i + 1], ch]);
                            lock.col_add_one();
                        }
                    }
                    _ => {
                        // ignore
                    }
                }
                i += 3;
                continue;
            }

            // if not arrow, handle normal keys
            match ch {
                CR | LF => {
                    // always '\n'
                    let ch = LF;

                    // echo
                    self.write(tty.clone(), &[ch]);

                    // push ch
                    let mut lock = self.echo_buf.lock();
                    lock.buffer.push(ch);

                    // copy echo buffer to read buffer
                    // FIXME: currently flush read_buf and push all data to read_buf
                    let len = lock.buffer.len();
                    for _ in 0..len {
                        let ch = lock.buffer.delete(0);
                        self.read_buf.push(ch);
                    }

                    // col set to 0
                    lock.col_clear();
                }
                BS | DEL => {
                    let mut lock = self.echo_buf.lock();
                    let col = lock.col();
                    let len = lock.buffer.len();
                    // can delete
                    if col > 0 {
                        if col == len {
                            self.write(tty.clone(), &[BS, SPACE, BS]);
                            lock.buffer.delete(col - 1);
                            lock.col_sub_one();
                        } else {
                            self.write(tty.clone(), &[BS, SPACE, BS]);
                            for i in col..len {
                                let ch = lock.buffer.see(i);
                                self.write(tty.clone(), &[ch]);
                            }
                            self.write(tty.clone(), &[SPACE]);
                            for _ in 0..(len - col + 1) {
                                self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], LEFT]);
                            }
                            lock.buffer.delete(col - 1);
                            lock.col_sub_one();
                        }
                    }
                }
                _ => {
                    let mut lock = self.echo_buf.lock();
                    let col = lock.col();
                    let len = lock.buffer.len();
                    if col == len {
                        self.write(tty.clone(), &[ch]);
                        lock.buffer.push(ch);
                        lock.col_add_one();
                    } else {
                        self.write(tty.clone(), &[ch]);
                        for i in col..len {
                            self.write(tty.clone(), &[lock.buffer.see(i)]);
                        }
                        for _ in 0..(len - col) {
                            self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], LEFT]);
                        }
                        lock.buffer.insert(ch, col);
                        lock.col_add_one();
                    }
                }
            }
            i += 1;
        }
    }

    /// kernel wants to write.
    pub fn write(&self, tty: Arc<TtyStruct>, buf: &[u8]) -> usize {
        // just call driver to putchar.
        let mut len = 0;
        let driver = tty.driver();
        for ch in buf {
            len += 1;
            (driver.ops.putchar)(*ch);
        }
        len
    }
}
