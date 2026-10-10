#[cfg(windows)]
pub(crate) fn decrypt(encrypted: &[u8]) -> Option<String> {
    use std::ffi::c_void;

    #[repr(C)]
    struct DataBlob {
        size: u32,
        data: *mut u8,
    }

    #[link(name = "Crypt32")]
    extern "system" {
        fn CryptUnprotectData(
            input: *const DataBlob,
            description: *mut *mut u16,
            entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut DataBlob,
        ) -> i32;
    }
    #[link(name = "Kernel32")]
    extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }

    unsafe {
        let input = DataBlob {
            size: encrypted.len() as u32,
            data: encrypted.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            size: 0,
            data: std::ptr::null_mut(),
        };
        if CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut output,
        ) == 0
        {
            return None;
        }
        let result = String::from_utf8_lossy(std::slice::from_raw_parts(
            output.data,
            output.size as usize,
        ))
        .into_owned();
        LocalFree(output.data.cast());
        (result.chars().count() > 20).then_some(result)
    }
}

#[cfg(not(windows))]
pub(crate) fn decrypt(_: &[u8]) -> Option<String> {
    None
}
