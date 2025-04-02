// SPDX-FileCopyrightText: 2025 ANSSI
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// TODO:
// - Add unit tests
// - Handle every returned Status

//use crate::println;
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
    pub fn retrieve_handle(label: ShmLabel) -> Result<u32, Status> {
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
    pub fn new(label: ShmLabel) -> Result<Self, Status> {
        let handle = Self::retrieve_handle(label)?;

        Ok(Shm {
            handle: Some(handle),
            label: Some(label),
            mapped_to: None,
            perms: None,
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
    /// * `to_task` - The task to map the shared memory to
    /// * `perms` - The permissions to set for the shared memory
    /// # Example
    /// ```
    /// let handle = shm.map(0xAA, SHMPermission::Read | SHMPermission::Write);
    /// ```
    /// # Errors
    /// * `Status::Denied` - If handle cannot be retrieved
    /// * `Status::Busy` - If the shared memory is already mapped
    /// * `Status::Invalid` - If the shared memory is invalid
    pub fn map(&mut self, to_task: u32, perms: u32) -> Result<Status, Status> {
        // Check if the shared memory is already mapped
        if self.mapped_to.is_some() {
            return Err(Status::Busy);
        }

        // Try to retrieve the handle if it is not already set
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
            Status::Ok => {
                self.mapped_to = None;
                self.perms = None;
                Ok(Status::Ok)
            }
            _ => Err(Status::Denied),
        }
    }
    pub fn set_creds(&mut self, to_task: u32, perms: u32) -> Result<Status, Status> {
        // Check if the shared memory is currently mapped
        if self.mapped_to.is_some() {
            return Err(Status::Busy);
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
        // Setting creedentials
        match sentry_uapi::syscall::shm_set_credential(handle, to_task, perms) {
            Status::Ok => {
                self.perms = Some(perms);
                Ok(Status::Ok)
            }
            Status::Busy => Err(Status::Busy),
            _ => Err(Status::Denied),
        }
    }
    /// Get the credentials of the shared memory
    /// # Returns
    /// * `u32` - The credentials of the shared memory
    /// # Example
    /// ```
    /// let creds = shm.get_credential();
    /// ```
    pub fn get_creds(&mut self) -> Result<u32, Status> {
        // Get crendentials from get_info
        let shm_info = self.get_info()?;
        Ok(shm_info.perms)
    }
    /// Check if the shared memory is mappable
    /// # Returns
    /// * `bool` - True if the shared memory is mappable, false otherwise
    /// # Example
    /// ```
    /// let is_mappable = shm.is_mappable();
    /// ```
    pub fn is_mappable(&mut self) -> bool {
        let shm_info = self.get_info();
        match shm_info {
            Ok(info) => {
                if info.perms & SHMPermission::Map as u32 == 0 {
                    self.perms = Some(info.perms);
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    }
    /// Check if the shared memory is readable
    /// # Returns
    /// * `bool` - True if the shared memory is readable, false otherwise
    /// # Example
    /// ```
    /// let is_readable = shm.is_readable();
    /// ```
    pub fn is_readable(&mut self) -> bool {
        let shm_info = self.get_info();
        match shm_info {
            Ok(info) => {
                if info.perms & SHMPermission::Read as u32 == 0 {
                    self.perms = Some(info.perms);
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    }
    /// Check if the shared memory is writable
    /// # Returns
    /// * `bool` - True if the shared memory is writable, false otherwise
    /// # Example
    /// ```
    /// let is_writable = shm.is_writable();
    /// ```
    pub fn is_writable(&mut self) -> bool {
        let shm_info = self.get_info();
        match shm_info {
            Ok(info) => {
                if info.perms & SHMPermission::Write as u32 == 0 {
                    self.perms = Some(info.perms);
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    }
    /// Check if the shared memory is transferable
    /// # Returns
    /// * `bool` - True if the shared memory is transferable, false otherwise
    /// # Example
    /// ```
    /// let is_transferable = shm.is_transferable();
    /// ```
    pub fn is_transferable(&mut self) -> bool {
        let shm_info = self.get_info();
        match shm_info {
            Ok(info) => {
                if info.perms & SHMPermission::Transfer as u32 == 0 {
                    self.perms = Some(info.perms);
                    return true;
                }
                false
            }
            Err(_) => false,
        }
    }
    /// Find the base address of the shared memory
    /// # Returns
    /// * `usize` - The base address of the shared memory
    /// # Example
    /// ```
    /// let base = shm.get_base();
    /// ```
    pub fn get_base(&mut self) -> Result<usize, Status> {
        // Get base from get_info
        let shm_info = self.get_info()?;
        Ok(shm_info.base)
    }
    /// Find the length of the shared memory
    /// # Returns
    /// * `usize` - The length of the shared memory
    /// # Example
    /// ```
    /// let len = shm.get_len();
    /// ```
    pub fn get_len(&mut self) -> Result<usize, Status> {
        // Get len from get_info
        let shm_info = self.get_info()?;
        Ok(shm_info.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_new() {
        let mut new_shm = Shm {
            handle: None,
            label: Some(0x00),
            mapped_to: None,
            perms: None,
        };
        let shm = new_shm.new(0xAA);
        assert!(shm.is_ok());
        let shm = shm.unwrap();
        assert_eq!(shm.label, Some(0xAA));
    }

    #[test]
    fn test_shm_map() {
        let mut new_shm = Shm {
            handle: None,
            label: Some(0xAA),
            mapped_to: None,
            perms: None,
        };
        let map_result = new_shm.map(0x01, SHMPermission::Read | SHMPermission::Write);
        assert!(map_result.is_ok());
        assert_eq!(new_shm.mapped_to, Some(0x01));
    }

    #[test]
    fn test_shm_map_unmap() {
        let mut new_shm = Shm {
            handle: None,
            label: Some(0xAA),
            mapped_to: Some(0x01),
            perms: None,
        };
        let map_result = new_shm.map(0x01, SHMPermission::Read | SHMPermission::Write);
        assert!(map_result.is_ok());
        let unmap_result = new_shm.unmap();
        assert!(unmap_result.is_ok());
        assert_eq!(new_shm.mapped_to, None);
        assert_eq!(new_shm.perms, None);
    }

    #[test]
    fn test_shm_get_info() {
        let mut new_shm = Shm {
            handle: None,
            label: Some(0xAA),
            mapped_to: None,
            perms: None,
        };
        let info_result = new_shm.get_info();
        assert!(info_result.is_ok());
        let info = info_result.unwrap();
        assert_eq!(info.label, 0xAA);
    }
}
