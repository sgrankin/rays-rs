use oidn_sys::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use failure::*;

pub struct Device {
    pub(crate) handle: OIDNDevice,
}

impl Device {
    pub fn new() -> Device {
        let handle = unsafe { oidnNewDevice(OIDN_DEVICE_TYPE_DEFAULT) };
        unsafe {
            oidnCommitDevice(handle);
        }
        Device { handle }
    }

    pub fn get_error(&mut self) -> Result<(), Error> {
        let mut message_ptr = ptr::null();
        let code =
            unsafe { oidnGetDeviceError(self.handle, &mut message_ptr as *mut *const c_char) };
        if code == OIDN_ERROR_NONE {
            Ok(())
        } else {
            bail!(unsafe { CStr::from_ptr(message_ptr).to_string_lossy().to_string() });
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            oidnReleaseDevice(self.handle);
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for Device {}

impl Clone for Device {
    fn clone(&self) -> Device {
        unsafe { oidnRetainDevice(self.handle) };
        Device { handle: self.handle }
    }
}
