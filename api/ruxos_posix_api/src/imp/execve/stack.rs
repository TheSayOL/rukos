use core::{mem::size_of, ptr::null_mut};

use ruxconfig::TASK_STACK_SIZE;

use crate::*;

const STACK_SIZE: usize = TASK_STACK_SIZE;
static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

#[derive(Debug)]
pub struct Stack {
    /// addr of the top byte of stack
    sp: usize,
    /// addr of stack start
    start: usize,
    /// stack start plus stack size
    end: usize,
}

impl Stack {
    // alloc a stack
    pub fn new() -> Self {
        let p = unsafe { STACK.as_ptr() };

        let start = p as usize;
        let sp = start + STACK_SIZE;
        let end = sp;

        Self { sp, start, end }
    }

    pub fn align(&mut self, align: usize) -> usize {
        self.sp -= self.sp % align;
        self.sp
    }

    pub fn push<T>(&mut self, thing: &[T], align: usize) -> usize {
        // move sp to right place
        let size = thing.len() * size_of::<T>();
        self.sp -= size;
        self.sp = self.align(align);

        // write data into stack
        let mut pt = self.sp as *mut T;
        unsafe {
            pt.copy_from_nonoverlapping(thing.as_ptr(), thing.len());
        }

        self.sp
    }
}
