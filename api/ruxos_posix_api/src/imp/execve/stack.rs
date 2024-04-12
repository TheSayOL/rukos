use core::{mem::size_of, ptr::null_mut};

use crate::*;

#[derive(Debug)]
pub struct Stack {
    sp: usize,
    start: usize,
    end: usize,
}

impl Stack {
    // alloc a stack
    pub fn new() -> Self {
        let size = config::TASK_STACK_SIZE; // 10M
        let prot = ctypes::PROT_READ | ctypes::PROT_WRITE;
        let flags = ctypes::MAP_ANONYMOUS | ctypes::MAP_PRIVATE;
        let p = sys_mmap(null_mut(), size as _, prot as _, flags as _, -1, 0);

        unsafe {
            let s = core::slice::from_raw_parts_mut(p as *mut u8, size);
            s.fill(0);
        }

        let start = p as usize;
        let sp = start + size;
        let end = sp;

        Self { sp, start, end }
    }

    pub fn align(&mut self, align: usize) -> usize {
        self.sp -= self.sp % align;
        self.sp
    }

    pub fn push<T: Copy>(&mut self, thing: alloc::vec::Vec<T>, align: usize) -> usize {
        let size = thing.len() * size_of::<T>();
        self.sp -= size;
        self.sp = self.align(align); // align 16B

        if self.sp < self.start {
            panic!("stack overflow");
        }

        let mut pt = self.sp as *mut T;
        unsafe {
            for t in thing {
                *pt = t;
                pt = pt.add(1);
            }
        }

        self.sp
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        sys_munmap(self.start as *mut _, self.end - self.start);
    }
}
