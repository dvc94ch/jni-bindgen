//! Common glue code between Rust and JNI, used in autogenerated jni-bindgen glue code.
//! 
//! See also the [Android JNI tips](https://developer.android.com/training/articles/perf-jni) documentation as well as the
//! [Java Native Interface Specification](https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/jniTOC.html).

// Re-export a few things such that we have a consistent name for them in autogenerated glue code wherever we go.
#[doc(hidden)] pub use ::std as std;
#[doc(hidden)] pub use ::jni_sys as jni_sys;

use jni_sys::*;
use lazy_static::*;

use std::ffi::*;
use std::ptr::*;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::*;

pub(crate) mod backends { // XXX: Might expose this to the end user in order to let them choose which backend to use...?
    use super::*;

    mod single_vm_backend;

    pub(crate) use single_vm_backend::*;
}

mod refs {
    use super::*;

    mod argument;
    mod global;
    mod local;
    mod ref_;

    pub use argument::*;
    pub use global::*;
    pub use local::*;
    pub use ref_::*;
}

mod __jni_bindgen;
mod array;
mod as_jvalue;
mod as_valid_jobject_and_env;
mod env;
mod gen_vm;
mod jchar_;
mod jni_type;
mod object_and_env;
mod string_chars;
mod throwable_type;
mod vm;

pub use array::*;
pub use as_jvalue::*;
pub use as_valid_jobject_and_env::*;
pub use env::*;
pub(crate) use gen_vm::*;
pub use jchar_::{jchar, *};
pub use jni_type::JniType;
pub use object_and_env::*;
pub use refs::*;
pub use string_chars::*;
pub use throwable_type::*;
pub use vm::*;



type VmBackend = backends::SingleVmBackend;
lazy_static! { // RwLock::new is not const
    static ref VMS : RwLock<VmBackend> = RwLock::new(VmBackend::new());
}

/// **Disable "unsafe-manual-jni-load-unload", or call from JNI_OnLoad, or there may be soundness issues!**
#[cfg(feature = "unsafe-manual-jni-load-unload")]
pub unsafe fn on_load(vm: *const JavaVM, _reserved: *const c_void) -> jint {
    VMS.write().unwrap().on_load(vm);
    JNI_VERSION_1_2
}

/// **Disable "unsafe-manual-jni-load-unload", or call from JNI_OnUnload, or there will be soundness issues!**
#[cfg(feature = "unsafe-manual-jni-load-unload")]
pub fn on_unload(vm: *const JavaVM, _reserved: *const c_void) {
    VMS.write().unwrap().on_unload(vm);
}

/// **Do not call!**  Automatically invoked by the JVM.  See "unsafe-manual-jni-load-unload" to override this behavior.
#[no_mangle] #[allow(non_snake_case)] #[cfg(not(feature = "unsafe-manual-jni-load-unload"))]
pub unsafe extern "system" fn JNI_OnLoad(vm: *const JavaVM, _reserved: *const c_void) -> jint {
    VMS.write().unwrap().on_load(vm);
    JNI_VERSION_1_2
}

/// **Do not call!**  Automatically invoked by the JVM.  See "unsafe-manual-jni-load-unload" to override this behavior.
#[no_mangle] #[allow(non_snake_case)] #[cfg(not(feature = "unsafe-manual-jni-load-unload"))]
pub extern "system" fn JNI_OnUnload(vm: *const JavaVM, _reserved: *const c_void) {
    VMS.write().unwrap().on_unload(vm);
}
