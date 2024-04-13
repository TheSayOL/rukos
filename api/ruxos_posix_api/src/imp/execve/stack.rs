use alloc::{vec, vec::Vec};

use ruxconfig::TASK_STACK_SIZE;

const STACK_SIZE: usize = TASK_STACK_SIZE;

#[derive(Debug)]
pub struct Stack {
    /// stack
    data: Vec<u8>,
    /// index of top byte of stack
    top: usize,
}

impl Stack {
    /// alloc a stack
    pub fn new() -> Self {
        let data = vec![0u8; STACK_SIZE];
        let top = STACK_SIZE;

        Self { data, top }
    }

    /// panic if overflow
    fn panic_if_of(&self) {
        if self.top > self.data.len() {
            panic!("sys_execve: stack overflow");
        }
    }

    /// move sp to align
    pub fn align(&mut self, align: usize) -> usize {
        self.top -= self.top % align;
        self.top
    }

    /// addr of top of stack
    pub fn sp(&self) -> usize {
        let start_addr = self.data.as_ptr() as usize;
        start_addr + self.top
    }

    /// push data to stack and return the addr of sp
    pub fn push<T>(&mut self, data: &[T], align: usize) -> usize {
        // move sp to right place
        let size = core::mem::size_of_val(data);
        self.top -= size;
        self.top = self.align(align);

        self.panic_if_of();

        // write data into stack
        let pt = self.sp() as *mut T;
        unsafe {
            pt.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }

        pt as usize
    }
}
