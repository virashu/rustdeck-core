use std::{
    alloc::{self, Layout},
    ffi::CString,
};

use rustdeck_common::proto::Arg;

pub enum SafeArg {
    Bool(Arg),
    Int(Arg),
    Float(Arg),
    String(Arg),
}

impl SafeArg {
    /// # Safety
    /// `Arg` is valid until the `SafeArg` value is dropped
    pub fn as_arg(&self) -> Arg {
        match self {
            Self::Bool(i) | Self::Int(i) | Self::Float(i) | Self::String(i) => {
                Arg { i: unsafe { i.i } }
            }
        }
    }
}

impl Drop for SafeArg {
    fn drop(&mut self) {
        match self {
            Self::Bool(arg) => unsafe {
                alloc::dealloc(arg.b.cast_mut().cast(), Layout::new::<bool>());
            },
            Self::Int(arg) => unsafe {
                alloc::dealloc(arg.i.cast_mut().cast(), Layout::new::<i32>());
            },
            Self::Float(arg) => unsafe {
                alloc::dealloc(arg.f.cast_mut().cast(), Layout::new::<f32>());
            },
            Self::String(arg) => unsafe {
                _ = CString::from_raw(arg.c.cast_mut());
            },
        }
    }
}
