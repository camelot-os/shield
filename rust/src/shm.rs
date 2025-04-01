// SPDX-FileCopyrightText: 2025 ANSSI
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// TODO:
// - Add unit tests
// - Handle every returned Status
// - Add a shm.is_readable() function
// - Add a shm.is_writable() function
// - .map() add the credentials
// - .map() returns a MappedShm obj that holds ummap

use crate::println;
use core::option::Option;
use sentry_uapi::copy_from_kernel;
use sentry_uapi::systypes::SHMPermission;
use uapi::systypes::shm::ShmInfo;
use uapi::systypes::Status;
use uapi::systypes::{ShmHandle, ShmLabel};

struct Shm {
    handle: Option<ShmHandle>,
    label: Option<ShmLabel>,
    mapped_to: Option<u32>,
    perms: Option<u32>,
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
                let new_handle = Self::retrieve_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        // Check if the shared memory is already mapped
        if self.mapped_to.is_some() {
            Err(Status::AlreadyMapped)
        } else {
            Ok(Shm {
                handle: Some(handle),
                label: Some(label),
                mapped_to: None,
                perms:None,
            })
        }
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
    pub fn get_info(&mut self) -> Result<ShmInfo, Status> {
        // Create a new ShmInfo structure
        let mut shm_info = ShmInfo {
            label: 0,
            handle: 0,
            base: 0,
            len: 0,
            perms: SHMPermission::Read.into(),
        };
        let handle = match self.handle {
            Some(h) => h,
            None => {
                // Try to retrieve the handle
                let label = match self.label {
                    Some(l) => l,
                    None => return Err(Status::Invalid),
                };
                let new_handle = Self::retrieve_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        sentry_uapi::syscall::shm_get_infos(handle);
        match copy_from_kernel(&mut shm_info) {
            Ok(Status::Ok) => Ok(shm_info),
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
    pub fn map(&mut self, to_task: u32, perms: u32) -> Result<Status, Status> {
        // Check if the shared memory is already mapped
        if self.mapped_to.is_some() {
            return Err(Status::Busy);
        }

        // Try to retrieve the handle
        let handle = match self.handle {
            Some(h) => h,
            None => {
                let label = match self.label {
                    Some(l) => l,
                    None => return Err(Status::Invalid),
                };
                let new_handle = Self::retrieve_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        // Setting creedentials
        match sentry_uapi::syscall::shm_set_credential(handle, to_task, perms) {
            Status::Ok => {
                self.perms = Some(perms);
            }
            Status::Busy => return Err(Status::Busy),
            _ => return Err(Status::Denied),
            
        }

        // Map the shared memory
        match sentry_uapi::syscall::map_shm(handle) {
            Status::Ok => {
                self.mapped_to = Some(to_task);
                Ok(Status::Ok)
            }
            _ => Err(Status::Denied),
        }
    }
    pub fn unmap(&mut self) -> Result<Status, Status> {
        // Check if the shared memory is currently mapped
        if self.mapped_to.is_none() {
            return Err(Status::Invalid);
        }
        let handle = match self.handle {
            Some(h) => h,
            None => {
                let label = match self.label {
                    Some(l) => l,
                    None => return Err(Status::Invalid),
                };
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
            mapped_to: 0x00,
            perms:(SHMPermission::Read | SHMPermission::Write).into(),
        };
        let shm_info = Shm::get_info(new_shm.label);
        assert!(shm_info.is_ok());
        let info = shm_info.unwrap();
        assert_eq!(info.label, new_shm.label);
    }
}
