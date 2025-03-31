// SPDX-FileCopyrightText: 2025 ANSSI
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// TODO: 
// - Shm: add a new method to get the handle (avoid syscall/copy)
// - Avoid matching inceptions :D
// - Add a test for the unmap function
// - Add a test for the map function
// - Handle every returned Status
// - Add a shm.is_readable() function
// - Add a shm.is_writable() function
// - .map() add the credentials
// - .map() returns a MappedShm obj that holds ummap

use sentry_uapi::systypes::SHMPermission;
use uapi::systypes::shm::ShmInfo;
use uapi::systypes::Status;
use uapi::systypes::{ShmHandle, ShmLabel};
use core::option::Option;
use crate::println;
use sentry_uapi::copy_from_kernel;

struct Shm {
    handle: Option<ShmHandle>,
    label: ShmLabel,
    is_mapped: bool,
}

impl Shm {
        /// Retrieve the handle of the shared memory
        /// # Arguments
        ///   * `label` - The label of the shared memory
        /// # Returns
        ///   * `u32` - The handle of the shared memory
        /// # Example
        /// ```
        /// let handle = Shm::retrieve_handle(0xAA);
        /// ```
        /// # Errors
        ///   * `Status::Denied` - If handle cannot be retrieved
        ///   * `Status::Ok` - If handle is retrieved
        
        fn retrieve_handle(label: ShmLabel) -> Result<u32, Status> {
            if sentry_uapi::syscall::get_shm_handle(label) != Status::Ok {
                return Err(Status::Denied);
            }
    
            let mut handle = 0_u32;
            match copy_from_kernel(&mut handle) {
                Ok(Status::Ok) => Ok(handle),
                _ => Err(Status::Denied),
            }
        }
    /// Create a new shared memory
    /// # Arguments
    ///   * `label` - The label of the shared memory
    /// # Returns
    ///   * `Shm` - The shared memory object
    /// # Example
    /// ```
    /// let shm = Shm::new(0xAA);
    /// ```
    /// # Errors
    ///   * `Status::Denied` - If handle cannot be retrieved
    
    pub fn new(&mut self, label: ShmLabel) -> Result<Self, Status> {
        let handle = match self.handle {
            Some(h) => h,
            None => {
                // Try to retrieve the handle
                let new_handle = Self::retrieve_handle(self.label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        Ok(Shm {
            handle: Some(handle),
            label,
            is_mapped: false,
        })
        
    }
    /// Retrieve info about the shared memory
    /// # Arguments
    /// * `label` - The label of the shared memory
    /// # Returns
    /// * `ShmInfo` - The info of the shared memory
    /// # Example
    /// ```
    /// let info = shm.get_info(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - If handle cannot be retrieved

    pub fn get_info(label: ShmLabel) -> Result<ShmInfo, Status> {
        // Create a new ShmInfo structure
        let mut shm_info = ShmInfo {
            label: 0,
            handle: 0,
            base: 0,
            len: 0,
            perms: SHMPermission::Read.into(),
        };

        match sentry_uapi::syscall::get_shm_handle(label) {
            Status::Ok => {
                let mut handle: u32 = 0_u32;
                match copy_from_kernel(&mut handle) {
                    Ok(Status::Ok) => {
                        sentry_uapi::syscall::shm_get_infos(label);
                        match copy_from_kernel(&mut shm_info) {
                            Ok(Status::Ok) => Ok(shm_info),
                            _ => Err(Status::Denied),
                        }
                    }
                    _ => Err(Status::Denied),
                }
            }
            _ => Err(Status::Denied),
        }
    }
    /// Map the shared memory by retrieving the handle
    /// # Arguments
    /// * `shm` - The label of the shared memory
    /// # Returns
    /// * `Status` - The status of the operation
    /// # Example
    /// ```
    /// let handle = shm.map(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - If handle cannot be retrieved
    ///
    pub fn map<'a>(&'a mut self, label: ShmLabel) -> Result<Status, Status> {
        // Check if the shared memory is already mapped
        if self.is_mapped {
            return Err(Status::Busy);
        }

        // Try to retrieve the handle
        let handle = match self.handle {
            Some(h) => h,
            None => {
                let new_handle = Self::retrieve_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };

        // Map the shared memory
        match sentry_uapi::syscall::map_shm(handle) {
            Status::Ok => {
                self.is_mapped = true;
                Ok(Status::Ok)
            }
            _ => Err(Status::Denied),
        }
    }
    pub fn unmap(&mut self, label: ShmLabel) -> Result<Status, Status> {
        if !self.is_mapped {
            return Err(Status::Invalid);
        }
        let handle = match self.handle {
            Some(h) => h,
            None => {
                let new_handle = Self::retrieve_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        match sentry_uapi::syscall::unmap_shm(handle) {
            Status::Ok => Ok(Status::Ok),
            _ => Err(Status::Denied),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_get_info() {
        let new_shm = Shm {
            handle: None,
            label: 0x00,
            is_mapped: false,
        };
        let shm_info = Shm::get_info(new_shm.label);
        assert!(shm_info.is_ok());
        let info = shm_info.unwrap();
        assert_eq!(info.label, new_shm.label);
    }
}
