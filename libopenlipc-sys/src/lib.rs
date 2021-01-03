#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("./bindings.rs");

use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub struct rLIPC {
    conn: *mut LIPC,
}

impl rLIPC {
    pub fn new() -> Result<Self, String> {
        let lipc;
        unsafe {
            lipc = LipcOpenNoName();
        }
        if lipc == 0 as *mut c_void {
            // FIXME: NULL
            return Err(String::from("Failed to open a connection!"));
        }
        return Ok(Self { conn: lipc });
    }

    pub fn subscribe(
        &self,
        service: &str,
        name: Option<&str>,
        callback: fn(&str, &str) -> (),
    ) -> Result<(), String> {
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

        let code;
        unsafe {
            /* We wait to cast to .as_ptr() here
             * For a pointer to be valid, the thing it points to must still be around.
             * For a value to exist past the expression it's introduced in, it must be bound to a variable.
             * When the variable disappears, the value does too.
             * We must store the CString for _service and c_name, then independently get pointers
             * *to* them
             */
            code = LipcSubscribeExt(
                self.conn,
                _service.as_ptr(),
                c_name,
                Some(ugly_callback),
                ptr as *mut c_void,
            );
        }
        match code {
            LIPCcode_LIPC_OK => Ok(()),
            _ => Err(format!(
                "Failed to subscribe: {}",
                rLIPC::code_to_string(code)
            )),
        }
    }

    pub fn get_str_prop(&self, service: &str, prop: &str) -> Result<String, String> {
        let mut handle: *mut c_char = std::ptr::null_mut();
        let handle_ptr: *mut *mut c_char = &mut handle;

        let service = CString::new(service).unwrap();
        let prop = CString::new(prop).unwrap();
        let code;
        unsafe {
            code = LipcGetStringProperty(self.conn, service.as_ptr(), prop.as_ptr(), handle_ptr);
        };

        if code != LIPCcode_LIPC_OK {
            return Err(rLIPC::code_to_string(code));
        }
        let val;
        unsafe {
            val = CStr::from_ptr(handle).to_str().unwrap().to_owned().clone();
            // Made a copy, we can now free() the string
            LipcFreeString(handle);
        }
        Ok(val)
    }
    pub fn get_int_prop(&self, service: &str, prop: &str) -> Result<i32, String> {
        let mut val: c_int = 0;
        let service = CString::new(service).unwrap();
        let prop = CString::new(prop).unwrap();
        let code;
        unsafe {
            code = LipcGetIntProperty(self.conn, service.as_ptr(), prop.as_ptr(), &mut val);
        };

        if code != LIPCcode_LIPC_OK {
            return Err(rLIPC::code_to_string(code));
        }
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
    (*f)(_name, _source);
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
