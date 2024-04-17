//! Allocation:
//!
//! The first thing a driver needs to do is to allocate a struct tty_driver.
//! This is done by tty_alloc_driver() (or __tty_alloc_driver()).
//! Next, the newly allocated structure is filled with information.
//!
//! The allocation routines expect a number of devices the driver can handle at most and flags.
//! Flags are those starting `TTY_DRIVER_`.
//!
//! When the driver is about to be freed, tty_driver_kref_put() is called.
//! It will decrements the reference count and if it reaches zero, the driver is freed.
//!
//! Registration:
//!
//! tty_driver can be registered using `tty_register_driver()`.
//! It is recommended to pass `TTY_DRIVER_DYNAMIC_DEV` in flags of `tty_alloc_driver()`.
//! If it is not passed, all devices are also registered during tty_register_driver().
//!
//! Registering Devices:
//!
//! Every TTY device shall be backed by a struct tty_port.
//! Usually, TTY drivers embed tty_port into device’s private structures.
//! The driver is recommended to use tty_port’s reference counting by tty_port_get() and tty_port_put().
//! The final put is supposed to free the tty_port including the device’s private struct.
//! Unless TTY_DRIVER_DYNAMIC_DEV was passed as flags to tty_alloc_driver(),
//! TTY driver should register every device discovered in the system (the latter is preferred).
//! This is performed by tty_register_device()
//! or by tty_register_device_attr() if the driver wants to
//! expose some information through struct attribute_group.
//! Both of them register index’th device and upon return, the device can be opened.
//! It is up to driver to manage free indices and choosing the right one.
//! The TTY layer only refuses to register more devices than passed to tty_alloc_driver().
//! When the device is opened, the TTY layer allocates struct tty_struct
//! and starts calling operations from tty_driver.ops.
//!
//! Linking Devices to Ports
//! As stated earlier, every TTY device shall have a struct tty_port assigned to it.
//! It must be known to the TTY layer at tty_driver.ops.install() at latest.
//! There are few helpers to link the two.
//! Ideally, the driver uses tty_port_register_device() or tty_port_register_device_attr()
//! instead of tty_register_device() and tty_register_device_attr() at the registration time.
//! This way, the driver needs not care about linking later on.
//!
//! If that is not possible, the driver still can link the tty_port to a specific index
//! before the actual registration by tty_port_link_device().
//!
//! If it still does not fit, tty_port_install() can be used from the tty_driver.ops.install hook as a last resort.
//! This is dedicated mostly for in-memory devices like PTY where tty_ports are allocated on demand.

#[derive(Debug)]
pub struct TtyDriverOps {
    // pub write: fn(buf: &[u8]) -> usize,
    pub putchar: fn(u8),
}

use crate::tty::TtyStruct;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};
use lazy_init::LazyInit;
use spin::mutex::Mutex;
use spin::RwLock;

pub(crate) static ALL_DRIVERS: LazyInit<RwLock<AllDrivers>> = LazyInit::new();

pub struct AllDrivers {
    inner: Vec<Arc<TtyDriver>>,
}

pub fn get_driver_by_index(index: usize) -> Option<Arc<TtyDriver>> {
    ALL_DRIVERS.read().get_driver(index)
}

impl AllDrivers {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    /// add driver.
    pub fn add_driver(&mut self, mut driver: TtyDriver) -> usize {
        let index = self.inner.len();
        driver.index = index;
        let arc = Arc::new(driver);
        self.inner.push(arc);
        index
    }

    pub fn get_driver(&self, index: usize) -> Option<Arc<TtyDriver>> {
        for driver in self.inner.iter() {
            if driver.index == index {
                return Some(driver.clone());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct TtyDriver {
    /// driver operations.
    pub ops: TtyDriverOps,

    /// devices the driver control.
    ttys: Mutex<Vec<Arc<TtyStruct>>>,

    /// driver index.
    index: usize,

    /// name.
    name: String,
}

impl TtyDriver {
    pub fn new(ops: TtyDriverOps, name: &str) -> Self {
        Self {
            ops,
            ttys: Mutex::new(vec![]),
            index: 0,
            name: name.to_string(),
        }
    }

    /// add a device, return its index, -1 means failure.
    fn add_one_device(&self, tty: Arc<TtyStruct>) -> isize {
        let index = self.ttys.lock().len();
        tty.set_index(index);
        self.ttys.lock().push(tty);
        index as _
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    /// get all devices' name
    pub fn get_all_device_names(&self) -> Vec<String> {
        let mut ret = vec![];
        for dev in self.ttys.lock().iter() {
            let name = dev.name();
            ret.push(name);
        }
        ret
    }

    ///
    pub fn get_device_by_name(&self, name: &str) -> Option<Arc<TtyStruct>> {
        for tty in self.ttys.lock().iter() {
            if tty.name() == name {
                return Some(tty.clone());
            }
        }
        None
    }

    pub fn get_device_by_index(&self, index: usize) -> Option<Arc<TtyStruct>> {
        let lock = self.ttys.lock();
        if let Some(dev) = lock.get(index) {
            return Some(dev.clone());
        }
        None
    }
}

/// called by driver to register itself.
///
/// return driver's index.
pub fn register_driver(ops: TtyDriverOps, name: &str) -> usize {
    let driver = TtyDriver::new(ops, name);
    ALL_DRIVERS.write().add_driver(driver)
}

/// called by driver to register device.
///
/// return device's index, -1 means failure.
pub fn register_device(driver_index: usize) -> isize {
    let driver = ALL_DRIVERS.read().get_driver(driver_index);
    let mut index = -1;
    if let Some(d) = driver {
        let tty = TtyStruct::new(d.clone(), crate::ldisc::LdiscIndex::NTty);
        let tty = Arc::new(tty);
        index = d.add_one_device(tty.clone());
        crate::tty::add_one_device(tty.clone());
    }
    index
}

// /// tty driver's operations.
// /// Define the interface between the low-level tty driver and the tty routines.
// /// These routines can be defined.
// /// Unless noted otherwise, they are optional, and can be filled in with a NULL pointer.
// #[derive(Default)]
// pub struct TtyOperations {
//     /// Return the tty device corresponding to idx,
//     /// NULL if there is not one currently in use and an ERR_PTR value on error.
//     /// Called under tty_mutex (for now!).
//     /// Optional method. Default behaviour is to use the self->ttys array.
//     lookup: Option<fn(_self: &TtyDriver, file: &File, idx: c_int) -> *mut TtyStruct>,

//     /// Install a new tty into the self’s internal tables.
//     /// Used in conjunction with lookup and remove methods.
//     /// Optional method. Default behaviour is to use the self->ttys array.
//     install: Option<fn(_self: &TtyDriver, tty: &TtyStruct) -> c_int>,

//     /// Remove a closed tty from the self’s internal tables.
//     /// Used in conjunction with lookup and remove methods.
//     /// Optional method. Default behaviour is to use the self->ttys array.
//     remove: Option<fn(_self: &TtyDriver, tty: &TtyStruct)>,

//     /// Called when a particular tty device is opened.
//     /// If this routine is not filled in, the attempted open will fail with ENODEV.
//     /// Required method. Called with tty lock held. May sleep.
//     open: Option<fn(tty: &TtyStruct, file: &File) -> c_int>,

//     /// Called when a particular tty device is closed.
//     /// At the point of return from this call the driver must make no further ldisc calls.
//     /// Remark: called even if the corresponding open() failed.
//     /// Required method. Called with tty lock held. May sleep.
//     close: Option<fn(tty: &TtyStruct, file: &File)>,

//     /// Called under the tty lock when a particular tty device is closed for the last time.
//     /// It executes before the tty resources are freed,
//     /// so may execute while another function holds a tty kref.
//     shutdown: Option<fn(tty: &TtyStruct)>,

//     /// Called asynchronously when a particular tty device is closed for the last time freeing up the resources.
//     /// This is actually the second part of shutdown for routines that might sleep.
//     cleanup: Option<fn(tty: &TtyStruct)>,

//     /// Called by the kernel to write `count` of characters from `buf` to the tty device.
//     /// The characters may come from user space or kernel space.
//     /// Return the number of characters actually accepted for writing.
//     /// May occur in parallel in special cases.
//     /// Because this includes panic paths drivers generally shouldn’t try and do clever locking here.
//     /// Optional: Required for writable devices. May not sleep.
//     write: Option<fn(tty: &TtyStruct, buf: *const c_uchar, count: c_int) -> c_int>,

//     /// Called by the kernel to write a single character `ch` to the tty device.
//     /// Kernel must call the `flush_chars()` (if defined) when it is done stuffing characters into the driver.
//     /// If there is no room in the queue, the character is ignored.
//     /// Optional: Kernel will use `write()` if not provided.
//     /// Do not call this function directly, call tty_put_char().
//     put_char: Option<fn(tty: &TtyStruct, ch: c_uchar) -> c_int>,

//     /// Called by the kernel after it has written a series of characters to the tty device using `put_char()`.
//     /// Optional. Do not call this function directly, call tty_driver_flush_chars().
//     flush_chars: Option<fn(tty: &TtyStruct)>,

//     /// Return the numbers of characters the tty driver will accept for queuing to be written.
//     /// This number is subject to change as output buffers get emptied, or if the output flow control is acted.
//     /// The ldisc is responsible for being intelligent about multi-threading of write_room/write calls
//     /// Required if write method is provided.
//     /// Do not call this function directly, call tty_write_room()
//     write_room: Option<fn(tty: &TtyStruct) -> c_uint>,

//     /// Return the number of characters in the device private output queue.
//     /// Used in tty_wait_until_sent() and for poll() implementation.
//     /// Optional: if not provided, it is assumed there is no queue on the device.
//     /// Do not call this function directly, call tty_chars_in_buffer().
//     chars_in_buffer: Option<fn(tty: &TtyStruct) -> c_uint>,

//     /// Allow the tty driver to implement device-specific ioctls.
//     /// If the ioctl number passed in cmd is not recognized by the driver, return ENOIOCTLCMD.
//     /// Optional.
//     ioctl: Option<fn(tty: &TtyStruct, cmd: c_uint, arg: c_ulong) -> c_int>,

//     ///  Implement ioctl processing for 32 bit process on 64 bit system.
//     /// Optional.
//     compat_ioctl: Option<fn(tty: &TtyStruct, cmd: c_uint, arg: c_ulong) -> c_long>,

//     /// Allow the tty driver to be notified when device’s termios settings have changed.
//     /// New settings are in tty->termios. Previous settings are passed in the `old`.
//     /// The API is defined such that the driver should return the actual modes selected.
//     /// This means that the driver should modify any bits in tty->termios it cannot fulfill to indicate the actual modes.
//     /// Optional. Called under the tty->termios_rwsem. May sleep.
//     set_termios: Option<fn(tty: &TtyStruct, old: &KTermios)>,

//     /// Notify the tty driver that input buffers for the ldisc are close to full,
//     /// and it should somehow signal that no more characters should be sent to the tty.
//     /// Serialization including with unthrottle() is the job of the ldisc layer.
//     /// Optional: Always invoke via tty_throttle_safe(). Called under the tty->termios_rwsem.
//     throttle: Option<fn(tty: &TtyStruct)>,

//     /// Notify the tty driver that it should signal that characters can now be sent to the tty without fear of overrunning the input buffers of the ldisc.
//     /// Optional. Always invoke via tty_unthrottle(). Called under the tty->termios_rwsem.
//     unthrottle: Option<fn(tty: &TtyStruct)>,

//     /// Notify the tty driver that it should stop outputting characters to the tty device.
//     /// Called with tty->flow.lock held. Serialized with start() method.
//     /// Optional. Always invoke via stop_tty().
//     stop: Option<fn(tty: &TtyStruct)>,

//     /// Notify the tty driver that it resumed sending characters to the tty device.
//     /// Called with tty->flow.lock held. Serialized with stop() method.
//     /// Optional. Always invoke via start_tty().
//     start: Option<fn(tty: &TtyStruct)>,

//     /// Notify the tty driver that it should hang up the tty device.
//     /// Optional. Called with tty lock held.
//     hangup: Option<fn(tty: &TtyStruct)>,

//     /// Request the tty driver to turn on/off BREAK status on the RS-232 port.
//     /// If `state` is -1, then the BREAK status should be turned on; if `state` is 0, off.
//     /// if this method is implemented, The high-level tty driver will handle the following ioctls: TCSBRK, TCSBRKP, TIOCSBRK, TIOCCBRK.
//     /// If the driver sets TTY_DRIVER_HARDWARE_BREAK in tty_alloc_driver(),
//     /// then the interface will also be called with actual times and the hardware is expected to do the delay work itself.
//     /// 0 and -1 are still used for on/off.
//     /// Optional: Required for TCSBRK/BRKP/etc. handling. May sleep.
//     break_ctl: Option<fn(tty: &TtyStruct, state: c_int) -> c_int>,

//     /// Discard device private output buffer.
//     /// Invoked on close, hangup, to implement TCOFLUSH ioctl and similar.
//     /// Optional: if not provided, it is assumed there is no queue on the device.
//     /// Do not call this function directly, call tty_driver_flush_buffer().
//     flush_buffer: Option<fn(tty: &TtyStruct)>,

//     /// Allow the tty driver to be notified when the device’s ldisc is being changed.
//     /// At the point this is done the discipline is not yet usable.
//     /// Optional. Called under the tty->ldisc_sem and tty->termios_rwsem.
//     set_ldisc: Option<fn(tty: &TtyStruct)>,

//     /// Wait until the device has written out all of the characters in its transmitter FIFO.
//     /// Or until timeout (in jiffies) is reached.
//     /// Optional: If not provided, the device is assumed to have no FIFO.
//     /// Usually correct to invoke via tty_wait_until_sent(). May sleep.
//     wait_until_sent: Option<fn(tty: &TtyStruct, timeout: c_int)>,

//     /// Used to send a high-priority XON/XOFF character (ch) to the tty device.
//     /// Optional: If not provided, then the write method is called under the tty->atomic_write_lock to keep it serialized with the ldisc.
//     send_xchar: Option<fn(tty: &TtyStruct, ch: c_char)>,

//     /// Used to obtain the modem status bits from the tty driver.
//     /// Optional: If not provided, then ENOTTY is returned from the TIOCMGET ioctl.
//     /// Do not call this function directly, call tty_tiocmget().
//     tiocmget: Option<fn(tty: &TtyStruct) -> c_int>,

//     /// Used to set the modem status bits to the tty driver.
//     /// First, clear bits should be cleared, then set bits set.
//     /// Optional: If not provided, then ENOTTY is returned from the TIOCMSET ioctl.
//     /// Do not call this function directly, call tty_tiocmset().
//     tiocmset: Option<fn(tty: &TtyStruct, set: c_uint, clear: c_uint) -> c_int>,

//     /// Called when a termios request is issued which changes the requested terminal geometry to `ws`.
//     /// Optional: the default action is to update the termios structure without error.
//     /// This is usually the correct behaviour.
//     /// Drivers should not force errors here if they are not resizable objects (e.g. a serial line).
//     /// See tty_do_resize() if you need to wrap the standard method in your own logic – the usual case.
//     resize: Option<fn(tty: &TtyStruct, ws: &WinSize) -> c_int>,

//     /// Called when the tty device receives a TIOCGICOUNT ioctl.
//     /// Passed a kernel structure icount to complete.
//     /// Optional: if not provided, ENOTTY will be returned.
//     get_icount: Option<fn(tty: &TtyStruct, icount: &SerialICounter) -> c_int>,

//     /// Called when the tty device receives a TIOCGSERIAL ioctl.
//     /// Passed a kernel structure `p` to complete.
//     /// Optional: if not provided, ENOTTY will be returned.
//     /// Do not call this function directly, call tty_tiocgserial().
//     get_serial: Option<fn(tty: &TtyStruct, p: &SerialStruct) -> c_int>,

//     /// Called when the tty device receives a TIOCSSERIAL ioctl.
//     /// Passed a kernel structure `p` to set the values from.
//     /// Optional: if not provided, ENOTTY will be returned.
//     /// Do not call this function directly, call tty_tiocsserial().
//     set_serial: Option<fn(tty: &TtyStruct, p: &SerialStruct) -> c_int>,

//     /// Called when the tty device file descriptor receives a fdinfo request from VFS (to show in /proc/<pid>/fdinfo/).
//     /// `m` should be filled with information.
//     /// Optional: if not provided, nothing is written to `m`.
//     /// Do not call this function directly, call tty_show_fdinfo().
//     show_fdinfo: Option<fn(tty: &TtyStruct, m: &SeqFile)>,

//     /// kgdboc support (Using kgdb, kdb and the kernel debugger internals).
//     /// This routine is called to initialize the HW for later use by calling poll_get_char or poll_put_char.
//     /// Optional: if not provided, skipped as a non-polling driver.
//     poll_init: Option<fn(tty: &TtyStruct, line: c_int, options: *mut c_char) -> c_int>,

//     /// kgdboc support (see poll_init).
//     /// driver should read a character from a tty identified by `line` and return it.
//     /// Optional: called only if poll_init provided.
//     poll_get_char: Option<fn(tty: &TtyStruct, line: c_int) -> c_int>,

//     /// kgdboc support (see poll_init).
//     /// driver should write character ch to a tty identified by line.
//     /// Optional: called only if poll_init provided.
//     poll_put_char: Option<fn(tty: &TtyStruct, line: c_int, ch: c_char)>,

//     /// Driver `driver` (cast to struct `tty_driver`) can show additional info in /proc/tty/driver/<driver_name>.
//     /// It is enough to fill in the information into m.
//     /// Optional: if not provided, no /proc entry created.
//     proc_show: Option<fn(m: &SeqFile, driver: *mut c_void) -> c_int>,
// }

// impl TtyOperations {
//     fn new() -> Self {
//         Self {
//             break_ctl: None,
//             chars_in_buffer: None,
//             cleanup: None,
//             close: None,
//             compat_ioctl: None,
//             flush_buffer: None,
//             flush_chars: None,
//             get_icount: None,
//             get_serial: None,
//             hangup: None,
//             install: None,
//             ioctl: None,
//             lookup: None,
//             open: None,
//             poll_get_char: None,
//             poll_init: None,
//             poll_put_char: None,
//             proc_show: None,
//             put_char: None,
//             remove: None,
//             resize: None,
//             send_xchar: None,
//             set_ldisc: None,
//             set_serial: None,
//             set_termios: None,
//             show_fdinfo: None,
//             shutdown: None,
//             start: None,
//             stop: None,
//             throttle: None,
//             tiocmget: None,
//             tiocmset: None,
//             unthrottle: None,
//             wait_until_sent: None,
//             write: None,
//             write_room: None,
//         }
//     }
// }

// /// Allocate this struct by tty_alloc_driver(),
// /// set up all the necessary members, and register by tty_register_driver().
// /// At last, the driver is torn down by calling tty_unregister_driver() followed by tty_driver_kref_put().
// /// The fields required to be set before calling tty_register_driver() include:
// /// driver_name, name, type, subtype, init_termios, and ops.
// pub struct TtyDriver {
//     /// set to TTY_DRIVER_MAGIC in __tty_alloc_driver()
//     // magic: c_int,

//     /// reference counting. Reaching zero frees all the internals and the driver.
//     // kref: Kref,

//     /// allocated/registered character /dev devices
//     // cdevs: Vec<*mut Cdev>,

//     /// modules owning this driver.
//     /// Used drivers cannot be rmmod’ed.
//     /// Automatically set by tty_alloc_driver().
//     // owner: Module,

//     /// used in /proc/tty
//     // driver_name: *const c_char,

//     /// used for constructing /dev node name
//     // name: *const c_char,

//     /// used as a number base for constructing /dev node name
//     // name_base: c_int,

//     /// major /dev device number (zero for autoassignment)
//     // major: c_int,

//     /// the first minor /dev device number
//     // minor_start: c_int,

//     /// number of devices allocated
//     // num: c_uint,

//     /// type of tty driver (TTY_DRIVER_TYPE_)
//     // _type: c_short,

//     /// subtype of tty driver (SYSTEM_TYPE_, PTY_TYPE_, SERIAL_TYPE_)
//     // subtype: c_short,

//     /// termios to set to each tty initially (e.g. tty_std_termios)
//     // init_termios: KTermios,

//     /// tty driver flags, c_ulong
//     // flags: TtyDriverFlag,

//     /// proc fs entry, used internally
//     // proc_entry: *mut ProcDirEntry,

//     /// array of active struct tty_struct, set by tty_standard_install()
//     // ttys: Mutex<Vec<Arc<TtyStruct>>>,

//     /// array of struct tty_port;
//     /// can be set during initialization by tty_port_link_device() and similar
//     ports: Mutex<Vec<TtyPort>>,

//     /// storage for termios at each TTY close for the next open
//     // termios: Vec<Arc<KTermios>>,

//     /// driver of the linked tty; only used for the PTY driver
//     // other: *mut TtyDriver,

//     /// pointer to driver’s arbitrary data
//     // driver_state: *mut c_void,

//     /// driver hooks for TTYs.
//     /// Set them using tty_set_operations().
//     /// Use struct tty_port helpers in them as much as possible.
//     ops: TtyOperations,
//     // /// list of all drivers
//     // tty_drivers: &Vec<TtyDriver>,
// }

// impl TtyDriver {
//     pub fn new(ops: TtyOperations) -> Self {
//         Self {
//             // ttys: vec![],
//             ports: Mutex::new(vec![]),
//             // termios: vec![],
//             ops,
//         }
//     }
//     pub fn add_port(&mut self, port: TtyPort) {
//         self.ports.lock().push(port);
//     }
// }

// static mut DRIVERS: Mutex<Vec<TtyDriver>> = Mutex::new(vec![]);

// fn drivers_push(driver: TtyDriver) {
//     unsafe { DRIVERS.lock().push(driver) };
// }

// /// allocate tty driver.
// /// This should not be called directly, some of the provided macros should be used instead.
// ///
// /// Use IS_ERR() and friends on retval.
// // pub fn tty_alloc_driver() -> TtyDriver {
// //     unimplemented!()
// // }

// /// Called by a tty driver to register itself.
// pub fn tty_register_driver(mut driver: TtyDriver) {
//     // auto register devices
//     let mut port = TtyPort::new();
//     port.link_device(1);

//     // link port and driver
//     driver.add_port(port);

//     // save driver
//     drivers_push(driver);
// }

// /// drop a reference to a tty driver
// /// The final put will destroy and free up the driver.
// // pub fn tty_driver_kref_put(driver: *mut TtyDriver) {
// //     unimplemented!()
// // }

// /// Called by a tty driver to unregister itself.
// // pub fn tty_unregister_driver(driver: *mut TtyDriver) {
// //     unimplemented!()
// // }

// /// register an individual tty device if the tty driver’s flags have the TTY_DRIVER_DYNAMIC_DEV bit set.
// /// If that bit is not set, this function should not be called by a tty driver.
// ///
// /// `index`: index for this tty device.
// /// `device`: a struct associated with this tty device. optional.
// ///
// /// Return: A pointer to the struct `device` for this tty device (or ERR_PTR(-EFOO) on error).
// // fn tty_register_device(
// //     driver: *mut TtyDriver,
// //     index: c_uint,
// //     device: Option<Device>,
// // ) -> *mut Device {
// //     unimplemented!()
// // }

// /// register an individual tty device if the tty driver’s flags have the TTY_DRIVER_DYNAMIC_DEV bit set.
// /// If that bit is not set, this function should not be called by a tty driver.
// ///
// /// `drvdata`: Driver data to be set to device.
// /// `attr_grp`: Attribute group to be set on device.
// // pub fn tty_register_device_attr(
// //     driver: *mut TtyDriver,
// //     index: c_uint,
// //     device: *mut Device,
// //     drvdata: *mut c_void,
// //     attr_grp: *const *const AttributeGroup,
// // ) -> *mut Device {
// //     unimplemented!()
// // }

// /// unregister a tty device.
// /// If a tty device is registered with a call to tty_register_device()
// /// then this function must be called when the tty device is gone.
// // pub fn tty_unregister_device(driver: *mut TtyDriver, index: c_uint) {
// //     unimplemented!()
// // }

// /// link tty and tty_port.
// /// Provide the tty layer with a link from a tty (specified by `index`) to a tty_port.
// /// Use this only if neither tty_port_register_device() nor tty_port_install() is used in the driver.
// /// Has to be called before tty_register_driver().
// // fn tty_port_link_device(port: TtyPort, driver: Arc<TtyDriver>, index: usize) -> TtyDevice {
// // }

// /// register tty device
// ///
// /// It is the same as tty_register_device_attr() except the provided port is linked to a concrete tty specified by index.
// /// Use this or tty_port_install() (or both).
// /// Call tty_port_link_device() as a last resort.
// ///
// /// `drvdata`: Driver data to be set to device.
// /// `attr_grp`: Attribute group to be set on device.
// // pub fn tty_port_register_device_attr(
// //     port: *mut TtyPort,
// //     driver: *mut TtyDriver,
// //     index: c_uint,
// //     device: *mut Device,
// //     drvdata: *mut c_void,
// //     attr_grp: *const *const AttributeGroup,
// // ) -> *mut Device {
// //     unimplemented!()
// // }

// /// register tty device
// ///
// /// Same as tty_register_device() except the `port` is linked to a tty specified by `index`.
// ///
// /// Use this or tty_port_install() (or both).
// /// Call tty_port_link_device() as a last resort.
// ///
// /// `device`: parent if exists, otherwise NULL
// // pub fn tty_port_register_device(mut port: TtyPort, driver: Arc<TtyDriver>, index: usize) {
// //     // port links device.
// //     port.link_device(index);
// //     // driver links device.
// // }

// /// Flags used for alloc
// enum TtyDriverFlag {
//     /// Requests the tty layer to reset the termios setting
//     /// when the last process has closed the device.
//     /// Used for PTYs, in particular.
//     TtyDriverResetTermios,

//     /// Indicates that the driver will guarantee not to set
//     /// any special character handling flags if this is set for the tty:
//     ///
//     /// (IGNBRK || (!BRKINT && !PARMRK)) && (IGNPAR || !INPCK)
//     ///
//     /// That is, if there is no reason for the driver to send notifications
//     /// of parity and break characters up to the line driver, it won’t do so.
//     /// This allows the line driver to optimize for this case if this flag is set.
//     /// (Note that there is also a promise, if the above case is true, not to signal overruns, either.)
//     TtyDriverRealRaw,

//     /// The individual tty devices need to be registered with a call to tty_register_device()
//     /// when the device is found in the system,
//     /// and unregistered with a call to tty_unregister_device()
//     /// so the devices will be shown up properly in sysfs.
//     ///
//     /// If not set, all `tty_driver.num` entries will be created
//     /// by the tty core in sysfs when tty_register_driver() is called.
//     ///
//     /// This is to be used by drivers that have tty devices that can appear and disappear
//     /// while the main tty driver is registered with the tty core.
//     TtyDriverDynamicDev,

//     /// Don’t use the standard arrays (tty_driver.ttys and tty_driver.termios),
//     /// instead use dynamic memory keyed through the devpts filesystem.
//     /// This is only applicable to the PTY driver.
//     TtyDriverDevptsMem,

//     /// Hardware handles break signals.
//     /// Pass the requested timeout to the tty_operations.break_ctl instead of using a simple on/off interface.
//     TtyDriverHardwareBreak,

//     /// Do not allocate structures which are needed per line for this driver (tty_driver.ports) as it would waste memory.
//     /// The driver will take care. This is only applicable to the PTY driver.
//     TtyDriverDynamicAlloc,

//     /// Do not create numbered /dev nodes. For example, create /dev/ttyprintk and not /dev/ttyprintk0.
//     /// Applicable only when a driver for a single tty device is being allocated.
//     TtyDriverUnnumberedNode,
// }
