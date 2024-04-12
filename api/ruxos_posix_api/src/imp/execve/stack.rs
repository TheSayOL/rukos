use alloc::vec;

use ruxconfig::TASK_STACK_SIZE;

const STACK_SIZE: usize = TASK_STACK_SIZE;

#[derive(Debug)]
pub struct Stack {
    /// addr of the top byte of stack
    sp: usize,
    /// addr of stack start
    start: usize,
}

impl Stack {
    // alloc a stack
    pub fn new() -> Self {
        let stack = vec![0u8; STACK_SIZE];

        let p = stack.as_ptr();

        let start = p as usize;
        let sp = start + STACK_SIZE;

        Self { sp, start }
    }

    /// panic if overflow
    fn panic_if_of(&self) {
        if self.sp < self.start {
            panic!("sys_execve: stack overflow");
        }
    }

    pub fn align(&mut self, align: usize) -> usize {
        self.sp -= self.sp % align;
        self.sp
    }

    pub fn push<T>(&mut self, data: &[T], align: usize) -> usize {
        // move sp to right place
        let size = core::mem::size_of_val(data);
        self.sp -= size;
        self.sp = self.align(align);

        self.panic_if_of();

        // write data into stack
        let pt = self.sp as *mut T;
        unsafe {
            pt.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }

        self.sp
    }
}
