//! struct `tty_struct` is allocated by the TTY layer upon the first open of the TTY device
//! and released after the last close.
//! The TTY layer passes this structure to most of struct tty_operation’s hooks.

use core::sync::atomic::AtomicUsize;

use alloc::{string::String, sync::Arc, vec, vec::Vec};
use lazy_init::LazyInit;
use spin::RwLock;

use crate::{
    driver::TtyDriver,
    ldisc::{LdiscIndex, TtyLdisc},
};

pub(crate) static ALL_DEVICES: LazyInit<RwLock<AllDevices>> = LazyInit::new();

pub(crate) struct AllDevices {
    inner: Vec<Arc<TtyStruct>>,
}

impl AllDevices {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
    pub fn get_device_by_name(&self, name: &str) -> Option<Arc<TtyStruct>> {
        for tty in self.inner.iter() {
            if tty.name() == name {
                return Some(tty.clone());
            }
        }
        None
    }
    pub fn get_device_by_index(&self, index: usize) -> Option<Arc<TtyStruct>> {
        for tty in self.inner.iter() {
            if tty.index.load(core::sync::atomic::Ordering::Relaxed) == index {
                return Some(tty.clone());
            }
        }
        None
    }

    pub fn get_all_device_names(&self) -> Vec<String> {
        let mut ret = vec![];
        for tty in self.inner.iter() {
            ret.push(tty.name())
        }
        ret
    }

    pub fn add_one_device(&mut self, tty: Arc<TtyStruct>) {
        self.inner.push(tty);
    }
}

pub fn get_device_by_name(name: &str) -> Option<Arc<TtyStruct>> {
    ALL_DEVICES.read().get_device_by_name(name)
}

pub fn get_device_by_index(index: usize) -> Option<Arc<TtyStruct>> {
    ALL_DEVICES.read().get_device_by_index(index)
}

pub fn get_all_device_names() -> Vec<String> {
    ALL_DEVICES.read().get_all_device_names()
}

pub fn add_one_device(tty: Arc<TtyStruct>) {
    ALL_DEVICES.write().add_one_device(tty)
}

#[derive(Debug)]
pub struct TtyStruct {
    /// driver index.
    driver: Arc<TtyDriver>,

    pub ldisc: Arc<TtyLdisc>,

    /// index of device.
    index: AtomicUsize,
}

impl TtyStruct {
    pub fn new(driver: Arc<TtyDriver>, ldisc_index: LdiscIndex) -> Self {
        Self {
            driver: driver.clone(),
            ldisc: crate::ldisc::new_ldisc(ldisc_index),
            index: AtomicUsize::new(0),
        }
    }

    pub fn set_index(&self, index: usize) {
        self.index
            .store(index, core::sync::atomic::Ordering::Relaxed);
    }

    pub fn name(&self) -> String {
        let driver = self.driver.clone();
        let mut name = driver.name();
        name.push(
            core::char::from_digit(
                self.index.load(core::sync::atomic::Ordering::Relaxed) as _,
                16,
            )
            .unwrap(),
        );
        name
    }

    pub fn driver(&self) -> Arc<TtyDriver> {
        self.driver.clone()
    }
}

// /// helper for termios setup.
// /// Initialise the termios structure for this tty.
// /// This runs under the tty_mutex currently so we can be relaxed about ordering.
// pub fn tty_init_termios(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// return tty naming.
// /// Convert a tty structure into a name.
// /// The name reflects the kernel naming policy
// /// and if udev is in use may not reflect user space
// /// Locking: None
// pub fn tty_name(tty: *mut TtyStruct) -> *const c_char {
//     unimplemented!()
// }

// /// get a tty reference
// /// Return a new reference to a tty object.
// /// The caller must hold sufficient locks/counts to ensure that their existing reference cannot go away
// pub fn tty_kref_get(tty: *mut TtyStruct) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// release a tty kref
// /// Release a reference to the tty device
// /// and if need be let the kref layer destruct the object for us.
// pub fn tty_kref_put(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// usual tty->ops->install
// /// If the driver overrides tty->ops->install,
// /// it still can call this function to perform the standard install operations.
// pub fn tty_standard_install(driver: *mut TtyDriver, tty: TtyStruct) -> c_int {
//     unimplemented!()
// }

// ///write one character to a tty
// /// Write one byte to the tty using the provided tty->ops->put_char() method if present.
// /// the specific put_char operation in the driver layer may go away soon.
// /// Don’t call it directly, use this method.
// /// Return the number of characters successfully output.
// pub fn tty_put_char(tty: *mut TtyStruct, ch: c_uchar) -> c_int {
//     unimplemented!()
// }

// /// propagate flow control
// /// Perform flow control to the driver.
// /// May be called on an already stopped device and will not re-call the tty_driver->stop() method.
// /// Used by both the line disciplines for halting incoming flow and by the driver.
// /// It may therefore be called from any context, may be under the tty atomic_write_lock but not always.
// /// Locking: flow.lock
// pub fn stop_tty(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// propagate flow control
// /// Start a tty that has been stopped if at all possible.
// /// If tty was previously stopped and is now being started,
// /// the tty_driver->start() method is invoked and the line discipline woken.
// /// Locking: flow.lock
// pub fn start_tty(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// request more data
// /// Internal and external helper for wakeups of tty.
// /// This function informs the line discipline
// /// if present that the driver is ready to receive more output data.
// pub fn tty_wakeup(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// trigger a hangup event
// /// A carrier loss (virtual or otherwise) has occurred on tty.
// /// Schedule a hangup sequence to run after this event.
// pub fn tty_hangup(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// process vhangup
// /// The user has asked via system call for the terminal to be hung up.
// /// We do this synchronously so that when the syscall returns the process is complete.
// /// That guarantee is necessary for security reasons.
// pub fn tty_vhangup(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// was tty hung up.
// /// `filp`: file pointer of tty.
// /// Return true if the tty has been subject to a vhangup or a carrier loss
// pub fn tty_hung_up_p(filp: *mut File) -> c_int {
//     unimplemented!()
// }

// /// resize event
// /// Update the termios variables and send the necessary signals to peform a terminal resize correctly.
// pub fn tty_do_resize(tty: *mut TtyStruct, ws: *mut WinSize) -> c_int {
//     unimplemented!()
// }

// /// Used in the tty_struct.flags.
// /// So that interrupts won’t be able to mess up the queues,
// /// `copy_to_cooked` must be atomic with respect to itself, as must tty->write.
// /// Thus, you must use the inline functions set_bit() and clear_bit() to make things atomic.
// enum TtyStructFlag {
//     /// Driver input is throttled.
//     /// The ldisc should call tty_driver.unthrottle() in order to resume reception
//     /// when it is ready to process more data (at threshold min).
//     TtyThrottled,

//     /// Cause all subsequent userspace read/write calls to fail, returning -EIO.
//     TtyIoError,

//     /// Device is a pty and the other side has closed.
//     TtyOtherClosed,

//     /// Exclusive open mode (a single opener).
//     TtyExclusive,

//     /// Cause the driver to call the tty_ldisc_ops.write_wakeup()
//     /// in order to resume transmission when it can accept more data.
//     TtyDoWriteWakeup,

//     // Indicate that a line discipline is open, for debugging.
//     TtyLdiscOpen,

//     /// Private to pty code, to implement TIOCSPTLCK/TIOCGPTLCK logic.
//     TtyPtyLock,

//     /// Prevent driver from splitting up writes into smaller chunks
//     /// (preserve write boundaries to driver).
//     TtyNoWriteSplit,

//     /// The TTY was hung up. This is set post tty_driver.hangup().
//     TtyHupped,

//     /// The TTY is in the process of hanging up to abort potential readers.
//     TtyHupping,

//     /// Line discipline for this TTY is being changed.
//     /// I/O should not block when this is set. Use tty_io_nonblock() to check.
//     TtyLdiscChanging,

//     /// Line discipline for this TTY was stopped.
//     /// No work should be queued to this ldisc.
//     TtyLdiscHalted,
// }

// /// All of the state associated with a tty while the tty is open.
// /// Persistent storage for tty devices is referenced here as struct tty_port `port` .
// pub struct TtyStruct {
//     /// magic value set early in alloc_tty_struct to TTY_MAGIC, for debugging purposes
//     magic: c_int,

//     /// reference counting by tty_kref_get() and tty_kref_put(),
//     /// reaching zero frees the structure
//     kref: Kref,

//     /// class device or NULL (e.g. ptys, serdev)
//     dev: *mut Device,

//     /// struct tty_driver operating this tty
//     driver: *mut TtyDriver,

//     /// struct tty_operations of driver for this tty
//     ops: *const TtyOperations,

//     /// index of this tty (e.g. to construct name like tty12)
//     index: c_int,

//     /// protects line discipline changes (ldisc) – lock tty not pty
//     ldisc_sem: LdSemaphore,

//     /// the current line discipline for this tty (n_tty by default)
//     ldisc: *mut TtyLdisc,

//     /// protects against concurrent writers,
//     /// i.e. locks write_cnt, write_buf and similar
//     atomic_write_lock: Mutex,

//     /// leftover from history (BKL -> BTM -> legacy_mutex),
//     /// protecting several operations on this tty
//     legacy_mutex: Mutex,

//     /// protects against concurrent tty_throttle_safe() and tty_unthrottle_safe() (but not tty_unthrottle())
//     throttle_mutex: Mutex,

//     /// protects termios and termios_locked
//     termios_rwsem: RWSemaphore,

//     /// protects winsize
//     winsize_mutex: Mutex,

//     /// termios for the current tty, copied from/to driver.termios
//     termios: KTermios,

//     /// locked termios (by TIOCGLCKTRMIOS and TIOCSLCKTRMIOS ioctls)
//     termios_locked: KTermios,

//     /// name of the tty constructed by tty_line_name() (e.g. ttyS3)
//     name: [c_char; 64],

//     /// bitwise OR of TTY_THROTTLED, TTY_IO_ERROR, …
//     flags: c_ulong,

//     /// count of open processes,
//     /// reaching zero cancels all the work for this tty and drops a kref too (but does not free this tty)
//     count: c_int,

//     /// size of the terminal “window” (cf. winsize_mutex)
//     winsize: WinSize,

//     /// flow settings grouped together, see also flow.unused
//     flow: Flow,

//     /// control settings grouped together, see also ctrl.unused
//     ctrl: Ctrl,

//     /// not controlled by the tty layer,
//     /// under driver’s control for CTS handling
//     hw_stopped: c_int,

//     /// bytes permitted to feed to ldisc without any being lost
//     receive_room: c_uint,

//     /// controls behavior of throttling, see tty_throttle_safe() and tty_unthrottle_safe()
//     flow_change: c_int,

//     /// link to another pty (master -> slave and vice versa)
//     link: *mut TtyStruct,

//     /// state for O_ASYNC (for SIGIO); managed by fasync_helper()
//     fasync: *mut FAsync,

//     /// concurrent writers are waiting in this queue until they are allowed to write
//     write_wait: WaitQueueHead,

//     /// readers wait for data in this queue
//     read_wait: WaitQueueHead,

//     /// normally a work to perform a hangup (do_tty_hangup());
//     /// while freeing the tty, (re)used to release_one_tty()
//     hangup_work: WorkStruct,

//     /// pointer to ldisc’s private data (e.g. to struct n_tty_data)
//     disc_data: *mut c_void,

//     /// pointer to driver’s private data (e.g. struct uart_state)
//     driver_data: *mut c_void,

//     /// protects tty_files list
//     files_lock: Spinlock,

//     /// list of (re)openers of this tty (i.e. linked struct tty_file_private)
//     tty_files: ListHead,

//     /// when set during close, n_tty processes only START & STOP chars
//     closing: c_int,

//     /// temporary buffer used during tty_write() to copy user data to
//     write_buf: *mut c_uchar,

//     /// count of bytes written in tty_write() to write_buf
//     write_cnt: c_int,

//     /// if the tty has a pending do_SAK, it is queued here
//     sak_work: WorkStruct,

//     /// persistent storage for this device (i.e. struct tty_port)
//     port: *mut TtyPort,
// }

// struct Flow {
//     /// lock for flow members
//     lock: Spinlock,
//     /// tty stopped/started by stop_tty()/start_tty()
//     stopped: bool,
//     /// tty stopped/started by TCOOFF/TCOON ioctls (it has precedence over flow.stopped)
//     tco_stopped: bool,
//     /// alignment for Alpha,
//     /// so that no members other than flow.* are modified by the same 64b word store.
//     /// The flow’s __aligned is there for the very same reason.
//     unused: [c_ulong; 0],
// }

// struct Ctrl {
//     /// lock for ctrl members
//     lock: Spinlock,
//     /// process group of this tty (setpgrp(2))
//     pgrp: *mut Pid,
//     /// session of this tty (setsid(2)).
//     /// Writes are protected by both ctrl.lock and legacy_mutex,
//     /// readers must use at least one of them.
//     session: *mut Pid,
//     /// packet mode status (bitwise OR of TIOCPKT_ constants)
//     pktstatus: c_uchar,
//     /// packet mode enabled
//     packet: bool,
//     /// alignment for Alpha, see flow.unused for explanation
//     unused: [c_ulong; 0],
// }
