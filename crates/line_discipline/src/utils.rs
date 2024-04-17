

// pub fn slice2str(slice:&[u8])->&str{
//     core::str::from_utf8(slice).unwrap()
// }






// use crate::termios::{File, IovIter, Kiocb, PollTableStruct, SerialICounter, WinSize, WorkStruct};
// use crate::driver::TtyDriver;
// use crate::tty::TtyStruct;
// use crate::types::{c_ssize_t, DevT};
// use core::ffi::{c_char, c_int, c_uint};


// /// Kopen
// /// Functions for opening a TTY from the kernelspace:

// /// closes tty opened by tty_kopen
// /// The final steps to release and free a tty device.
// /// It is the same as tty_release_struct() except that it also resets TTY_PORT_KOPENED flag on tty->port.
// pub fn tty_kclose(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// open a tty device for kernel
// /// Open tty exclusively for kernel.
// /// Performs the driver lookup, makes sure it’s not already opened
// /// and performs the first-time tty initialization.
// ///
// /// Claims the global tty_mutex to serialize:
// /// - concurrent first-time tty initialization
// /// - concurrent tty driver removal w/ lookup
// /// - concurrent tty removal from driver table
// ///
// /// Return the locked initialized tty_struct
// pub fn tty_kopen_exclusive(device: DevT) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// Opens an already existing tty for in-kernel use.
// /// Doesn’t ensure to be the only user.
// ///
// /// Locking: identical to tty_kopen().
// pub fn tty_kopen_shared(device: DevT) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// Exported Internal Functions

// /// return dev_t for device name
// ///
// /// Convert device names like ttyS0 into dev_t like (4, 64) or (188, 1).
// /// If no corresponding driver is registered then return -ENODEV.
// ///
// /// `name`: user space name of device under /dev.
// ///
// /// Locking: acquire tty_mutex to protect the tty_drivers list from being modified while traversed.
// pub fn tty_dev_name_to_number(name: *const c_char, number: *mut DevT) -> c_int {
//     unimplemented!()
// }

// /// release a tty struct
// ///
// /// Performs the final steps to release and free a tty device.
// /// It is roughly the reverse of tty_init_dev().
// ///
// /// `idx`: index of tty
// pub fn tty_release_struct(tty: *mut TtyStruct, idx: c_int) {
//     unimplemented!()
// }

// /// get tty statistics
// ///
// /// Get a copy of the tty’s icount statistics.
// ///
// /// Locking: none (up to the driver)
// pub fn tty_get_icount(tty: *mut TtyStruct, icount: *mut SerialICounter) -> c_int {
//     unimplemented!()
// }

// /// Internal Functions

// /// free a disused tty
// ///
// /// Free the write buffers, tty queue and tty memory itself.
// ///
// /// Locking: none. Must be called after tty is unused.
// pub fn free_tty_struct(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// free file->private_data
// ///
// /// Used only for fail path handling when tty_add_file was not called yet.
// pub fn tty_free_file(file: *mut File) {
//     unimplemented!()
// }

// /// find device of a tty.
// ///
// /// Return a tty driver, given a device number and passes back the index number.
// ///
// /// `index`: returns the index of the tty
// ///
// /// Locking: caller must hold tty_mutex
// pub fn get_tty_driver(device: DevT, index: *const c_int) -> *mut TtyDriver {
//     unimplemented!()
// }

// /// Release a redirect on a pty if present
// ///
// /// Available to the pty code so if the master closes, if the slave is a redirect, it can release the redirect.
// pub fn tty_release_redirect(tty: *mut TtyStruct) -> *mut File {
//     unimplemented!()
// }

// /// Actual handler for hangup events
// ///
// /// Called by a “kworker” kernel thread.
// /// That is process synchronous but doesn’t hold any locks,
// /// so we need to make sure we have the appropriate locks for what we’re doing.
// ///
// /// The hangup event clears any pending redirections onto the hung up device.
// /// It ensures future writes will error and it does the needed line discipline hangup and signal delivery.
// /// The tty object itself remains intact.
// ///
// /// `exit_session`: if non-zero, signal all foreground group processes.
// ///
// /// Locking:
// /// - BTM
// ///     - redirect lock for undoing redirection
// ///     - file list lock for manipulating list of ttys
// ///     - tty_ldiscs_lock from called functions
// ///     - termios_rwsem resetting termios data
// ///     - tasklist_lock to walk task list for hangup event
// ///         - ->siglock to protect ->signal/->sighand
// pub fn __tty_hangup(tty: *mut TtyStruct, exit_session: c_int) {
//     unimplemented!()
// }

// /// Perform a vhangup on the current controlling tty
// pub fn tty_vhangup_self() {
//     unimplemented!()
// }

// /// hangup session leader exit
// ///
// /// The session leader is exiting and hanging up its controlling terminal.
// /// Every process in the foreground process group is signalled SIGHUP.
// /// Do this synchronously so that when the syscall returns the process is complete, for security reasons.
// pub fn tty_vhangup_session(tty: *mut TtyStruct) {
//     unimplemented!()
// }



// /// read method for tty device files.
// ///
// /// Perform the read system call function on this terminal device.
// /// Checks for hung up devices before calling the line discipline method.
// ///
// /// `iocb`: kernel I/O control block
// /// `to`: destination for the data read
// ///
// /// Locking: Locks the line discipline internally while needed. Multiple read calls may be outstanding in parallel.
// pub fn tty_read(iocb: *mut Kiocb, to: *mut IovIter) -> c_ssize_t {
//     unimplemented!()
// }

// /// write a message to a certain tty, not just the console.
// ///
// /// Used for messages that need to be redirected to a specific tty.
// /// Don’t put it into the syslog queue right now maybe in the future if really needed.
// /// Must still hold the BTM and test the CLOSING flag for the moment.
// pub fn tty_write_message(tty: *mut TtyStruct, msg: *mut c_char) {
//     unimplemented!()
// }

// /// write method for tty device file.
// ///
// /// Write data to a tty device via the line discipline.
// ///
// /// `iocb`: kernel I/O control block.
// /// `from`: IovIter with data to write.
// ///
// /// Locking:
// /// Locks the line discipline as required
// /// Writes to the tty driver are serialized by the atomic_write_lock
// /// and are then processed in chunks to the device.
// /// The line discipline write method will not be invoked in parallel for each device.
// pub fn tty_write(iocb: *mut Kiocb, from: *mut IovIter) -> c_ssize_t {
//     unimplemented!()
// }

// /// send priority character.
// ///
// /// Send a high priority character to the tty even if stopped.
// ///
// /// Locking: none for xchar method, write ordering for write method.
// pub fn tty_send_xchar(tty: *mut TtyStruct, ch: c_char) -> c_int {
//     unimplemented!()
// }

// /// Generate a name from a driver reference and write it to the output buffer.
// ///
// /// `p`: output buffer of at least 6 bytes
// ///
// /// Locking: None
// pub fn pty_line_name(driver: *mut TtyDriver, index: c_int, p: *mut c_char) {
//     unimplemented!()
// }

// /// Generate a name from a driver reference and write it to the output buffer.
// ///
// /// `p`: output buffer of at least 7 bytes
// ///
// /// Locking: None
// pub fn tty_line_name(driver: *mut TtyDriver, index: c_int, p: *mut c_char) -> c_ssize_t {
//     unimplemented!()
// }

// /// find an existing tty, if any
// ///
// /// If not found, return NULL
// /// or ERR_PTR() if the driver lookup() method returns an error.
// ///
// /// Locking: tty_mutex must be held. If the tty is found, bump the tty kref.
// pub fn tty_driver_lookup_tty(
//     driver: *mut TtyDriver,
//     file: *mut File,
//     idx: c_int,
// ) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// Install a tty object into the driver tables.
// /// The `tty->index` will be set by the time this is called.
// /// Should ensure any needed additional structures are allocated and configured.
// ///
// /// Locking: tty_mutex for now
// pub fn tty_driver_install_tty(driver: *mut TtyDriver, tty: *mut TtyStruct) -> c_int {
//     unimplemented!()
// }

// /// Remove a tty object from the driver tables.
// /// The `tty->index` field will be set by the time this is called.
// ///
// /// Locking: tty_mutex for now
// pub fn tty_driver_remove_tty(driver: *mut TtyDriver, tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// fast re-open of an open tty.
// /// Re-opens on master ptys are not allowed and return -EIO.
// ///
// /// Locking: Caller must hold tty_lock
// ///
// /// Return 0 on success, -errno on error.
// pub fn tty_reopen(tty: *mut TtyStruct) -> c_int {
//     unimplemented!()
// }

// /// initialise a tty device.
// /// This may not be a “new” clean device but could also be an active device.
// /// The pty drivers require special handling because of this.
// ///
// /// Locking:
// /// Called under the tty_mutex, which protects us from the tty struct or driver itself going away.
// ///
// /// On exit the tty device has the line discipline attached and a reference count of 1.
// /// If a pair was created for pty/tty use and the other was a pty master then it too has a reference count of 1.
// ///
// /// WSH 06/09/97: Rewritten to remove races and properly clean up after a failed open.
// /// The new code protects the open with a mutex, so it’s really quite straightforward.
// /// The mutex locking can probably be relaxed for the (most common) case of reopening a tty.
// pub fn tty_init_dev(driver: *mut TtyDriver, idx: c_int) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// Sync flush all works belonging to tty (and the ‘other’ tty).
// pub fn tty_flush_works(tty: *mut TtyStruct) {
//     unimplemented!()
// }

// /// release tty structure memory
// /// Releases memory associated with a tty structure, and clears out the driver table slots.
// /// Called when a device is no longer in use.
// /// Also called when setup of a device fails.
// ///
// /// `work`: work of tty we are obliterating
// ///
// /// Locking:
// /// takes the file list lock internally when working on the list of ttys that the driver keeps.
// ///
// /// Called from a work queue so that the driver private cleanup ops can sleep (needed for USB at least)
// pub fn release_one_tty(work: *mut WorkStruct) {
//     unimplemented!()
// }

// /// Release both tty and a possible linked partner (think pty pair),
// /// and decrement the refcount of the backing module.
// ///
// /// Locking:
// /// tty_mutex takes the file list lock internally when working on the list of ttys that the driver keeps.
// pub fn release_tty(tty: *mut TtyStruct, idx: c_int) {
//     unimplemented!()
// }

// /// Performs some paranoid checking before true release of the tty.
// /// This is a no-op unless TTY_PARANOIA_CHECK is defined.
// pub fn tty_release_checks(tty: *mut TtyStruct, idx: c_int) -> c_int {
//     unimplemented!()
// }

// pub struct Inode;

// /// vfs callback for close
// ///
// /// Called the last time each file handle is closed that references this tty.
// /// There may however be several such references.
// ///
// /// Locking:
// /// Takes BKL. See tty_release_dev().
// ///
// /// Even releasing the tty structures is a tricky business.
// /// We have to be very careful that the structures are all released at the same time,
// /// as interrupts might otherwise get the wrong pointers.
// ///
// /// WSH 09/09/97: rewritten to avoid some nasty race conditions that could lead to double frees or releasing memory still in use.
// pub fn tty_release(inode: *mut Inode, filp: *mut File) -> c_int {
//     unimplemented!()
// }

// /// get locked tty of the current task iff `device` is /dev/tty
// ///
// /// Performs a re-open of the current task’s controlling tty.
// /// We cannot return driver and index like for the other nodes because devpts will not work then.
// /// It expects inodes to be from devpts FS.
// pub fn tty_open_current_tty(device: DevT, filp: *mut File) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// lookup a tty driver for a given device file
// ///
// /// If returned value is not erroneous,
// /// the caller should decrement the refcount by tty_driver_kref_put().
// ///
// /// `index`: index for the device in the return driver
// ///
// /// Locking: tty_mutex protects get_tty_driver()
// ///
// /// return driver for this inode (with increased refcount)
// pub fn tty_lookup_driver(device: DevT, filp: *mut File, index: *mut c_int) -> *mut TtyDriver {
//     unimplemented!()
// }

// /// open a tty device.
// ///
// /// Performs the driver lookup, checks for a reopen,
// /// or otherwise performs the first-time tty initialization.
// ///
// /// Claims the global tty_mutex to serialize:
// /// - concurrent first-time tty initialization
// /// - concurrent tty driver removal w/ lookup
// /// - concurrent tty removal from driver table
// ///
// /// Return the locked initialized or re-opened tty_struct
// pub fn tty_open_by_driver(device: DevT, filp: *mut File) -> *mut TtyStruct {
//     unimplemented!()
// }

// /// open a tty device.
// ///
// /// tty_open() and tty_release() keep up the tty count
// /// that contains the number of opens done on a tty.
// /// We cannot use the inode-count, as different inodes might point to the same tty.
// /// Open-counting is needed for pty masters,
// /// as well as for keeping track of serial lines: DTR is dropped when the last close happens.
// /// (This is not done solely through tty->count, now. - Ted 1/27/92)
// ///
// /// The termios state of a pty is reset on the first open so that settings don’t persist across reuse.
// ///
// /// Locking:
// /// - tty_mutex protects tty, tty_lookup_driver() and tty_init_dev().
// /// - tty->count should protect the rest.
// /// - ->siglock protects ->signal/->sighand
// ///
// /// Note:
// /// the tty_unlock/lock cases without a ref are only safe due to tty_mutex
// pub fn tty_open(inode: *mut Inode, filp: *mut File) -> c_int {
//     unimplemented!()
// }

// type PollT = c_uint;

// /// check tty status.
// ///
// /// Call the line discipline polling method to obtain the poll status of the device.
// ///
// /// Locking: locks called line discipline but ldisc poll method may be re-entered freely by other callers.
// pub fn tty_poll(filp: *mut File, wait: *mut PollTableStruct) -> PollT {
//     unimplemented!()
// }

// /// Fake input to a tty device.
// /// Does the necessary locking and input management.
// ///
// /// FIXME: does not honour flow control ??
// ///
// /// Locking:
// /// - Called functions take tty_ldiscs_lock
// /// - current->signal->tty check is safe without locks
// pub fn tiocsti(tty: *mut TtyStruct, p: *mut c_char) -> c_int {
//     unimplemented!()
// }

// /// implement window query ioctl.
// /// Copies the kernel idea of the window size into the user buffer.
// ///
// /// Locking: tty->winsize_mutex is taken to ensure the winsize data is consistent.
// pub fn tiocgwinsz(tty: *mut TtyStruct, arg: *mut WinSize) -> c_int {
//     unimplemented!()
// }

// /// implement window size set ioctl
// ///
// /// Copies the user idea of the window size to the kernel.
// /// Traditionally this is just advisory information
// /// but for the Linux console it actually has driver level meaning and triggers a VC resize.
// ///
// /// Locking:
// /// Driver dependent.
// /// The default do_resize method takes the tty termios mutex and ctrl.lock.
// /// The console takes its own lock then calls into the default method.
// pub fn tiocswinsz(tty: *mut TtyStruct, arg: *mut WinSize) -> c_int {
//     unimplemented!()
// }

// /// Allow the administrator to move the redirected console device.
// ///
// /// `file`: the file to become console.
// ///
// /// Locking: uses redirect_lock to guard the redirect information
// pub fn tioccons(file: *mut File) -> c_int {
//     unimplemented!()
// }

// /// Set the line discipline according to user request.
// ///
// /// Locking: see tty_set_ldisc(), this function is just a helper
// pub fn tiocsetd(tty: *mut TtyStruct, p: *mut c_int) -> c_int {
//     unimplemented!()
// }

// /// Get the line discipline id directly from the ldisc.
// ///
// /// Locking: waits for ldisc reference (in case the line discipline is changing or the tty is being hungup)
// pub fn tiocgetd(tty: *mut TtyStruct, p: *mut c_int) -> c_int {
//     unimplemented!()
// }

// /// Perform a timed break on hardware that lacks its own driver level timed break functionality.
// ///
// /// `duration`: timeout in ms
// ///
// /// Locking:
// /// tty->atomic_write_lock serializes
// pub fn send_break(tty: *mut TtyStruct, duration: c_uint) -> c_int {
//     unimplemented!()
// }

// /// get the modem status bits from the tty driver if the feature is supported.
// ///
// /// Return -ENOTTY if it is not available.
// ///
// /// Locking: none (up to the driver)
// pub fn tty_tiocmget(tty: *mut TtyStruct, p: *mut c_int) -> c_int {
//     unimplemented!()
// }

// /// Set the modem status bits from the tty driver if the feature is supported.
// ///
// /// Return -ENOTTY if it is not available.
// ///
// /// `cmd`: command - clear bits, set bits or set all
// /// `p`: pointer to desired bits
// ///
// /// Locking: none (up to the driver)
// pub fn tty_tiocmset(tty: *mut TtyStruct, cmd: c_uint, p: *mut c_uint) -> c_int {
//     unimplemented!()
// }

// /// Allocates and initializes a tty structure.
// ///
// /// `idx`: minor of the tty
// ///
// /// Locking: none - tty in question is not exposed at this point
// pub fn alloc_tty_struct(driver: *mut TtyDriver, idx: c_int) -> *mut TtyStruct {
//     unimplemented!()
// }
