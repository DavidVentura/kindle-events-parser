#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!("./bindings.rs");

use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

pub struct rLIPC {
    conn: *mut LIPC,
}

macro_rules! code_to_result {
    ($value:expr) => {
        if $value == LIPCcode_LIPC_OK {
            Ok(())
        } else {
            Err(format!(
                "Failed to subscribe: {}",
                rLIPC::code_to_string($value)
            ))
        }
    };
}

impl rLIPC {
    /// Returns a new LIPC client if a connection was successful
    /// Connects to the LIPC bus with no name.
    pub fn new() -> Result<Self, String> {
        let lipc;
        unsafe {
            lipc = LipcOpenNoName();
        }
        if lipc == (std::ptr::null_mut() as *mut c_void) {
            return Err(String::from("Failed to open a connection!"));
        }
        return Ok(Self { conn: lipc });
    }

    /// Register a callback for events broadcasted by `service`. Optionally,
    /// you can filter to a single event by providing `name`.
    ///
    /// For callback, we pass (source, name).
    ///
    /// # Examples
    ///
    /// ```
    /// let r = rLIPC.new();
    /// r.subscribe("com.lab126.powerd", Some("battLevel"), |_, _| ());
    /// // You will only get updates about battLevel in the callback
    /// ```
    ///
    /// ```
    /// let r = rLIPC.new();
    /// r.subscribe("com.lab126.powerd", None, |_, _| ());
    /// // You will get updates all power related events (screen on, off, etc)
    /// ```
    pub fn subscribe<F>(&self, service: &str, name: Option<&str>, callback: F) -> Result<(), String>
    where
        F: Fn(&str, &str) + Send,
    {
        let _service = CString::new(service).unwrap();

        let owned;
        let c_name = match name {
            None => std::ptr::null(),
            Some(_name) => {
                owned = CString::new(_name).unwrap();
                owned.as_ptr()
            }
        };

        let boxed_fn: Box<dyn FnMut(&str, &str) + Send> = Box::new(callback) as _;
        let double_box = Box::new(boxed_fn);
        let ptr = Box::into_raw(double_box);
        /*
         * You can't pass a fn directly to C -- you can however pass a `Box::into_raw`
         * This box however is of dynamic size and loses metadata -- so it's not easy to free later
         * The other box (boxed_fn) is a fat pointer (which we can't pass to C) but it keeps
         * metadata
         * So we pass a thin pointer (into_raw) to a fat pointer (<dyn FnMut..>) to C
         * then we have to undo this in the callback
         */

        let result;
        unsafe {
            /* We wait to cast to .as_ptr() here
             * For a pointer to be valid, the thing it points to must still be around.
             * For a value to exist past the expression it's introduced in, it must be bound to a variable.
             * When the variable disappears, the value does too.
             * We must store the CString for _service and c_name, then independently get pointers
             * *to* them
             */
            result = code_to_result!(LipcSubscribeExt(
                self.conn,
                _service.as_ptr(),
                c_name,
                Some(ugly_callback),
                ptr as *mut c_void,
            ));
        }
        result
    }

    /// Get the current value of a string property
    /// ```
    /// let r = rLIPC.new();
    /// let reader_status = r.get_str_prop("com.lab126.acxreaderplugin", "allReaderData").unwrap();
    /// // reader_status would be a string containing JSON
    /// ```
    pub fn get_str_prop(&self, service: &str, prop: &str) -> Result<String, String> {
        let mut handle: *mut c_char = std::ptr::null_mut();
        let handle_ptr: *mut *mut c_char = &mut handle;

        let service = CString::new(service).unwrap();
        let prop = CString::new(prop).unwrap();
        unsafe {
            code_to_result!(LipcGetStringProperty(
                self.conn,
                service.as_ptr(),
                prop.as_ptr(),
                handle_ptr
            ))?;
        };

        let val;
        unsafe {
            val = CStr::from_ptr(handle).to_str().unwrap().to_owned().clone();
            // Made a copy, we can now free() the string
            LipcFreeString(handle);
        }
        Ok(val)
    }

    /// Get the current value of an int property
    /// ```
    /// let r = rLIPC.new();
    /// let reader_status = r.get_int_prop("com.lab126.powerd", "battLevel").unwrap();
    /// // reader_status will contain the battery charge % (ie: 75).
    /// ```
    pub fn get_int_prop(&self, service: &str, prop: &str) -> Result<i32, String> {
        let mut val: c_int = 0;
        let service = CString::new(service).unwrap();
        let prop = CString::new(prop).unwrap();
        unsafe {
            code_to_result!(LipcGetIntProperty(
                self.conn,
                service.as_ptr(),
                prop.as_ptr(),
                &mut val
            ))?;
        };

        Ok(val)
    }

    fn code_to_string(code: u32) -> String {
        unsafe {
            let cstr = CStr::from_ptr(LipcGetErrorString(code));
            return String::from(cstr.to_str().unwrap());
        }
    }
}

unsafe extern "C" fn ugly_callback(
    _: *mut LIPC,
    name: *const c_char,
    event: *mut LIPCevent,
    data: *mut c_void,
) -> LIPCcode {
    let source = LipcGetEventSource(event);
    let _name = CStr::from_ptr(name).to_str().unwrap();
    let _source = CStr::from_ptr(source).to_str().unwrap();
    let f = data as *mut Box<dyn FnMut(&str, &str) + Send>;
    (*f)(_source, _name);
    return 0;
}

impl Drop for rLIPC {
    fn drop(&mut self) {
        unsafe {
            LipcClose(self.conn);
        }
        println!("Disconnected");
    }
}

unsafe impl Sync for rLIPC {}
