#![allow(non_camel_case_types)]

use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub union io_status_block_u {
    pub Status: i32,
    pub Pointer: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct io_status_block {
    pub u: io_status_block_u,
    pub Information: usize,
}

pub type io_apc_routine = unsafe extern "system" fn(
    ApcContext: *const c_void,
    IoStatusBlock: *const io_status_block,
    Reserved: u32,
);
