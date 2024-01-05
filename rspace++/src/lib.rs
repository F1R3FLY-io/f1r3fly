pub mod diskconc;
pub mod diskseq;
pub mod memconc;
pub mod memseq;
pub mod rspace;
pub mod rtypes;
pub mod setup;

use prost::Message;
use rspace::RSpace;
use rtypes::rtypes::{Commit, Retrieve};
use serde_json;
use std::ffi::{c_char, CStr, CString};

#[repr(C)]
pub struct Space {
    rspace: RSpace<rtypes::rtypes::Retrieve, rtypes::rtypes::Commit>,
}

#[no_mangle]
pub extern "C" fn space_new() -> *mut Space {
    Box::into_raw(Box::new(Space {
        rspace: RSpace::create().unwrap(),
    }))
}

// Verb Set 1
#[no_mangle]
pub extern "C" fn space_get_once_durable_concurrent(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_once_durable_concurrent(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            // let result_string = "";
            // let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            // c_string.into_raw()
            std::ptr::null()
        }

        // &result as *const OptionResult // return type *const OptionResult for Rust. ?? for Scala

        // Box::into_raw(Box::new(result)) // return type *mut Option<OptionResult> for Rust. ?? for Scala

        // 40 as *const i32 // return type *const i32 for Rust. Int for Scala

        // let hello_string = "Hello from Rust!".to_string();
        // let c_string = std::ffi::CString::new(hello_string).expect("Failed to create CString");
        // c_string.into_raw() // return type *const c_char for Rust. String for Scala

        // hello_string.as_ptr() as *const u8 // return type *const u8 for Rust. Pointer, Unit, or Long for Scala
    }
}

#[no_mangle]
pub extern "C" fn space_get_once_non_durable_concurrent(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_once_non_durable_concurrent(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_get_once_durable_sequential(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_once_durable_sequential(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_get_once_non_durable_sequential(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_once_non_durable_sequential(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

// Verb Set 2
#[no_mangle]
pub extern "C" fn space_get_always_durable_concurrent(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_always_durable_concurrent(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_get_always_non_durable_concurrent(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_always_non_durable_concurrent(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_get_always_durable_sequential(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_always_durable_sequential(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_get_always_non_durable_sequential(
    rspace: *mut Space,
    rdata_ptr: *const u8,
    rdata_len: usize,
) -> *const c_char {
    unsafe {
        let rdata_buf = std::slice::from_raw_parts(rdata_ptr, rdata_len);
        let rdata = Retrieve::decode(rdata_buf).unwrap();

        let result_option = (*rspace).rspace.get_always_non_durable_sequential(rdata);

        if result_option.is_some() {
            let result = result_option.unwrap();
            let result_string = serde_json::to_string(&result).unwrap();
            let c_string = std::ffi::CString::new(result_string).expect("Failed to create CString");
            c_string.into_raw()
        } else {
            std::ptr::null()
        }
    }
}

// Verb Set 3
#[no_mangle]
pub extern "C" fn space_put_once_durable_concurrent(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_once_durable_concurrent(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char // return type *const *const c_char for Rust. Array[String] for Scala
        } else {
            std::ptr::null()
        }

        // Box::into_raw(Box::new(result_option)) return type *mut Option<Vec<OptionResult>> for Rust. ?? for Scala
    }
}

#[no_mangle]
pub extern "C" fn space_put_once_non_durable_concurrent(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_once_non_durable_concurrent(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_put_once_durable_sequential(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_once_durable_sequential(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_put_once_non_durable_sequential(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_once_non_durable_sequential(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

// Verb Set 4
#[no_mangle]
pub extern "C" fn space_put_always_durable_concurrent(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_always_durable_concurrent(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_put_always_non_durable_concurrent(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_always_non_durable_concurrent(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_put_always_durable_sequential(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_always_durable_sequential(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn space_put_always_non_durable_sequential(
    rspace: *mut Space,
    cdata_ptr: *const u8,
    cdata_len: usize,
) -> *const *const c_char {
    unsafe {
        let cdata_buf = std::slice::from_raw_parts(cdata_ptr, cdata_len);
        let cdata = Commit::decode(cdata_buf).unwrap();

        let result_option = (*rspace).rspace.put_always_non_durable_sequential(cdata);

        if result_option.is_some() {
            let results = result_option.unwrap();
            let mut result_array: Vec<String> = vec![];

            for res in results {
                let result_string = serde_json::to_string(&res).unwrap();
                result_array.push(result_string);
            }

            let mut ptrs: Vec<_> = result_array
                .into_iter()
                .map(|s| {
                    CString::new(s)
                        .expect("Failed to create CString")
                        .into_raw()
                })
                .collect();
            ptrs.push(std::ptr::null_mut());
            let boxed_ptrs = ptrs.into_boxed_slice();

            Box::into_raw(boxed_ptrs) as *const *const c_char
        } else {
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn is_empty(rspace: *mut Space) -> bool {
    unsafe { (*rspace).rspace.is_empty() }
}

#[no_mangle]
pub extern "C" fn space_print(rspace: *mut Space, channel: *const c_char) -> () {
    unsafe {
        let channel_str = CStr::from_ptr(channel).to_str().unwrap();
        (*rspace).rspace.print_store(channel_str)
    }
}

#[no_mangle]
pub extern "C" fn space_clear(rspace: *mut Space) -> () {
    unsafe {
        (*rspace).rspace.clear_store();
    }
}
