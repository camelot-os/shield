// SPDX-FileCopyrightText: 2025 ANSSI
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// TODO: 
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

use crate::println;
use sentry_uapi::copy_from_kernel;

struct Shm<'a> {
    handle: &'a core::option::Option<ShmHandle>,
    label: ShmLabel,
}

impl Shm<'_> {
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
        match sentry_uapi::syscall::get_shm_handle(label) {
            Status::Ok => {
                let mut handle = 0_u32;
                match copy_from_kernel(&mut handle) {
                    Ok(Status::Ok) => {
                        sentry_uapi::syscall::map_shm(handle);
                        match copy_from_kernel(&mut handle) {
                            Ok(Status::Ok) => {
                                println!("Shared Memory {:#X} mapped.", handle);
                                Ok(Status::Ok)
                            }
                            Err(Status::Denied) => {
                                println!("Shared memoru mapping denied.");
                                Err(Status::Denied)
                            }
                            _ => Err(Status::Denied),
                        }
                    }
                    _ => Err(Status::Denied),
                }
            }
            _ => Err(Status::Denied),
        }
    }
    pub fn unmap(label: ShmLabel) -> Result<Status, Status> {
        match sentry_uapi::syscall::unmap_shm(label) {
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
            handle: &None,
            label: 0x00,
        };
        let shm_info = Shm::get_info(new_shm.label);
        assert!(shm_info.is_ok());
        let info = shm_info.unwrap();
        assert_eq!(info.label, new_shm.label);
    }
}
