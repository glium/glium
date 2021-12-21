use super::Context;

/// Describes an error preventing the retrieval of the uuid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UuidError {
    /// EXT_external_objects is not supported by the driver
    ExtensionNotPresent,
}

impl std::fmt::Display for UuidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            UuidError::ExtensionNotPresent => "EXT_external_objects is not supported by the driver",
        };

        f.write_str(desc)
    }
}

impl std::error::Error for UuidError {}

impl Context {
    /// Returns the UUID of the driver currently being used by this context.
    ///
    /// Useful to ensure compatibility when sharing resources with an external API.
    pub fn driver_uuid(&self) -> Result<[u8; 16], UuidError> {
        if !self.extensions.gl_ext_semaphore && !self.extensions.gl_ext_memory_object {
            return Err(UuidError::ExtensionNotPresent);
        }
        let mut data = [0u8; 16];
        unsafe {
            self.gl.GetUnsignedBytevEXT(crate::gl::DRIVER_UUID_EXT, data.as_mut_ptr())
        };
        Ok(data)
    }

    /// Returns the UUIDs of the devices being used by this context.
    ///
    /// Useful to ensure compatibility when sharing resources with an external API.
    pub fn device_uuids(&self) -> Result<Vec<[u8; 16]>, UuidError> {
        if !self.extensions.gl_ext_semaphore && !self.extensions.gl_ext_memory_object {
            return Err(UuidError::ExtensionNotPresent);
        }
        let mut n = std::mem::MaybeUninit::<i32>::uninit();
        let n = unsafe {
            self.gl.GetIntegerv(crate::gl::NUM_DEVICE_UUIDS_EXT, n.as_mut_ptr());
            n.assume_init()
        };

        let mut res = Vec::with_capacity(n as usize);
        for i in 0..n {
            let mut data = [0u8; 16];
            unsafe {
                self.gl.GetUnsignedBytei_vEXT(
                    crate::gl::DEVICE_UUID_EXT,
                    i as u32,
                    data.as_mut_ptr(),
                )
            };
            res.push(data);
        }

        Ok(res)
    }
}
