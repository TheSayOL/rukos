/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::{imp::fd_ops::get_file_like, sys_getpgid};
use axerrno::LinuxError;
use core::ffi::{c_int, c_uchar, c_uint};

/// IOCTL oprations
pub const TCGETS: usize = 0x5401;
pub const TIOCGPGRP: usize = 0x540F;
pub const TIOCSPGRP: usize = 0x5410;
pub const TIOCGWINSZ: usize = 0x5413;
pub const FIONBIO: usize = 0x5421;
pub const FIOCLEX: usize = 0x5451;

#[derive(Clone, Copy, Default)]
pub struct ConsoleWinSize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

const NCCS: usize = 19;
const ICANON: usize = 2;
const ECHO: usize = 8;

#[derive(Debug)]
#[repr(C)]
struct Termios {
    c_iflag: c_uint,       /* input mode flags */
    c_oflag: c_uint,       /* output mode flags */
    c_cflag: c_uint,       /* control mode flags */
    c_lflag: c_uint,       /* local mode flags */
    c_line: c_uchar,       /* line discipline */
    c_cc: [c_uchar; NCCS], /* control characters */
    c_ispeed: c_uint,      /* input speed */
    c_ospeed: c_uint,      /* output speed */
}

/// ioctl implementation,
/// currently only support fd = 1
pub fn sys_ioctl(fd: c_int, request: usize, data: usize) -> c_int {
    debug!("sys_ioctl <= fd: {}, request: {}", fd, request);
    syscall_body!(sys_ioctl, {
        match request {
            FIONBIO => {
                unsafe {
                    get_file_like(fd)?.set_nonblocking(*(data as *const i32) > 0)?;
                }
                Ok(0)
            }
            TIOCGWINSZ => {
                let winsize = data as *mut ConsoleWinSize;
                unsafe {
                    *winsize = ConsoleWinSize::default();
                }
                Ok(0)
            }
            TCGETS => {
                warn!("ioctl: tty TCGETS");
                let data = data as *const u8 as *mut Termios;
                unsafe {
                    const ICRNL: usize = 0x100;
                    (*data).c_iflag = ICRNL as _; //  ICRNL | IXON

                    const ISIG: usize = 1;
                    const ECHOE: usize = 0x10;
                    const ECHOK: usize = 0x20;
                    const ECHOCTL: usize = 0x200;
                    const ECHOKE: usize = 0x800;
                    const IEXTEN: usize = 0x8000;
                    (*data).c_lflag =
                        (ISIG | ICANON | ECHO | ECHOE | ECHOK | ECHOCTL | ECHOKE | IEXTEN) as _;

                    const OPOST: usize = 1;
                    const ONLCR: usize = 8;
                    (*data).c_oflag = (OPOST | ONLCR) as _;

                    // .c_cflag = B38400 | CS8 | CREAD | HUPCL,

                    // .c_cc = INIT_C_CC,
                    (*data).c_cc[2] = 0o177;

                    (*data).c_ispeed = 38400;
                    (*data).c_ospeed = 38400;
                    warn!("termios {:?}", *data);
                }

                Ok(0)
            }
            TIOCSPGRP => {
                warn!("stdout pretend to be tty");
                Ok(0)
            }
            TIOCGPGRP => {
                warn!("stdout TIOCGPGRP, pretend to be have a tty process group.");
                unsafe {
                    *(data as *mut u32) = sys_getpgid(0) as _;
                }
                Ok(0)
            }
            FIOCLEX => Ok(0),
            _ => Err(LinuxError::EINVAL),
        }
    })
}
