#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_compression_decompression() {
        unsafe {
            let device = oidnNewDevice(OIDN_DEVICE_TYPE_DEFAULT);
            oidnRetainDevice(device);
            oidnReleaseDevice(device);
            oidnReleaseDevice(device);
        }
    }
}
