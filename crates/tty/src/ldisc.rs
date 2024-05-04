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

use alloc::sync::Arc;
use spinlock::SpinNoIrq;

use crate::{
    buffer::{EchoBuffer, TtyBuffer},
    tty::TtyStruct,
};

const LF: u8 = b'\n';
const CR: u8 = b'\r';

const DEL: u8 = b'\x7f';
const BS: u8 = b'\x08';

const SPACE: u8 = b' ';

/// escape
const ESC: u8 = 27;
/// [
const LEFT_BRACKET: u8 = 91;

const ARROW_PREFIX: [u8; 2] = [ESC, LEFT_BRACKET];

// const UP: u8 = 65;
// const DOWN: u8 = 66;
const RIGHT: u8 = 67;
const LEFT: u8 = 68;

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
pub struct TtyLdisc {
    /// chars for tty_read()
    read_buf: TtyBuffer,

    /// chars echoing on the screen.
    echo_buf: SpinNoIrq<EchoBuffer>,

    /// tmp receive buffer
    rec_buf: TtyBuffer,
}

impl TtyLdisc {
    pub fn new() -> Self {
        Self {
            read_buf: TtyBuffer::new(),
            echo_buf: SpinNoIrq::new(EchoBuffer::new()),
            rec_buf: TtyBuffer::new(),
        }
    }
}

/// line discipline operations
impl TtyLdisc {
    pub fn name(&self) -> &str {
        "N_TTY"
    }

    pub fn num(&self) -> isize {
        LdiscIndex::NTty as _
    }

    /// called by kernel.
    /// send all data of read buffer to kernel.
    pub fn read(&self, buf: &mut [u8]) -> usize {
        let read_buf = &self.read_buf;

        // get len of this reading
        let len = buf.len().min(read_buf.len());

        // return if nothing can be read
        if len == 0 {
            return 0;
        }

        // copy data from read_buf to `buf`
        for i in 0..len {
            buf[i] = read_buf.see(i);
        }

        // flush
        read_buf.flush();

        // return len of reading
        len
    }

    /// called by driver.
    /// receive data from driver.
    /// process characters and echo.
    /// in irq.
    pub fn receive_buf(&self, tty: Arc<TtyStruct>, buf: &[u8]) {
        let rec_buf = &self.rec_buf;
        // add to receive buffer
        for ch in buf {
            rec_buf.push(*ch);
        }

        // process each char in receive buffer
        while rec_buf.len() > 0 {
            let ch = rec_buf.see(0);

            // if char may be arrow char
            if ch == ARROW_PREFIX[0] {
                // no enough len, just break, waitting for next time
                if rec_buf.len() < 3 {
                    break;
                }

                // enough len, but not a arrow char, just ignore
                if rec_buf.see(1) != ARROW_PREFIX[1] {
                    rec_buf.delete(0);
                    rec_buf.delete(0);
                    break;
                }

                // it is an arrow char, get it
                rec_buf.delete(0);
                rec_buf.delete(0);
                let ch = rec_buf.delete(0);

                // deal with arrow char
                match ch {
                    LEFT => {
                        let mut lock = self.echo_buf.lock();
                        // if can go left
                        if lock.col() > 0 {
                            self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], ch]);
                            lock.col_sub_one();
                        }
                    }
                    RIGHT => {
                        let mut lock = self.echo_buf.lock();
                        // if can go right
                        if lock.col() < lock.len() {
                            self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], ch]);
                            lock.col_add_one();
                        }
                    }
                    _ => {
                        // it is UP/DOWN, just ignore
                    }
                }
            // not a arrow char, handle it as a normal char
            } else {
                let ch = rec_buf.delete(0);
                match ch {
                    CR | LF => {
                        // always '\n'
                        let ch = LF;

                        // echo
                        self.write(tty.clone(), &[ch]);

                        // push this char to echo buffer
                        let mut lock = self.echo_buf.lock();
                        lock.buffer.push(ch);

                        // copy echo buffer to read buffer
                        // FIXME: currently will push all data to read_buf
                        let len = lock.buffer.len();
                        for _ in 0..len {
                            self.read_buf.push(lock.buffer.delete(0));
                        }

                        // echo buffer's column is set to 0
                        lock.col_clear();
                    }
                    BS | DEL => {
                        let mut lock = self.echo_buf.lock();
                        let col = lock.col();
                        let len = lock.buffer.len();
                        // if can delete
                        if col > 0 {
                            // perform a backspace
                            self.write(tty.clone(), &[BS, SPACE, BS]);

                            // if cursor is not on the rightmost
                            if col != len {
                                for i in col..len {
                                    let ch = lock.buffer.see(i);
                                    self.write(tty.clone(), &[ch]);
                                }
                                self.write(tty.clone(), &[SPACE]);
                                for _ in 0..(len - col + 1) {
                                    self.write(
                                        tty.clone(),
                                        &[ARROW_PREFIX[0], ARROW_PREFIX[1], LEFT],
                                    );
                                }
                            }

                            // modify echo buffer
                            lock.buffer.delete(col - 1);
                            lock.col_sub_one();
                        }
                    }
                    _ => {
                        // process normal chars.
                        let mut echo_buf = self.echo_buf.lock();
                        let col = echo_buf.col();
                        let len = echo_buf.buffer.len();

                        // echo
                        self.write(tty.clone(), &[ch]);

                        // if cursor is not on the rightmost
                        if col != len {
                            for i in col..len {
                                self.write(tty.clone(), &[echo_buf.buffer.see(i)]);
                            }
                            for _ in 0..(len - col) {
                                self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], LEFT]);
                            }
                        }

                        // modify echo buffer
                        echo_buf.buffer.insert(ch, col);
                        echo_buf.col_add_one();
                    }
                }
            }
        }
    }

    /// called by kernel.
    /// send data from kernel to driver.
    pub fn write(&self, tty: Arc<TtyStruct>, buf: &[u8]) -> usize {
        let mut len = 0;
        let driver = tty.driver();
        for ch in buf {
            len += 1;
            // call driver's putchar method
            (driver.ops.putchar)(*ch);
        }
        len
    }
}
