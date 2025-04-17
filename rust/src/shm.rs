// SPDX-FileCopyrightText: 2025 Stephane N (ANSSI)
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// TODO:
// - Add unit tests

use core::marker::PhantomData;
use core::option::Option;
use core::prelude::rust_2024::Err;
use sentry_uapi::copy_from_kernel;
use sentry_uapi::systypes::SHMPermission;
use uapi::systypes::Status;
use uapi::systypes::shm::ShmInfo;
use uapi::systypes::{ShmHandle, ShmLabel};

// Structs representing the state of the shared memory
// These are used as markers to indicate whether the shared memory is mapped or not
pub struct Mapped;
pub struct Unmapped;

// Trait SentryShm fot both States <Mapped, Unmapped>
pub trait SentryShm {
    type Mapped;
    type Unmapped;
    fn get_handle(label: ShmLabel) -> Result<u32, Status>;
    fn new(label: ShmLabel) -> Result<Self::Unmapped, Status>;
    fn get_info(&mut self) -> Result<ShmInfo, Status>;
    fn set_creds(&mut self, to_task: u32, perms: u32) -> Result<Status, Status>;
    fn get_creds(&mut self) -> Result<u32, Status>;
    fn is_mappable(&mut self) -> bool;
    fn is_readable(&mut self) -> bool;
    fn is_writable(&mut self) -> bool;
    fn is_transferable(&mut self) -> bool;
    fn get_base(&mut self) -> Result<usize, Status>;
    fn get_len(&mut self) -> Result<usize, Status>;
}

// Trait containing .map() for Shm<Unmapped>
pub trait Mappable {
    type Mapped;
    type Unmapped;
    fn map(&mut self, to_task: u32) -> Result<Self::Mapped, Status>;
}

// Trait containing .unmap() for Shm<Mapped>
pub trait Unmappable {
    type Mapped;
    type Unmapped;
    fn unmap(&mut self) -> Result<Self::Unmapped, Status>;
}

// PhantomData is used to indicate the state of the shared memory
pub struct Shm<State> {
    handle: Option<ShmHandle>,
    label: Option<ShmLabel>,
    mapped_to: Option<u32>,
    perms: Option<u32>,
    _marker: PhantomData<State>,
}

impl SentryShm for Shm<Unmapped> {
    type Mapped = Shm<Mapped>;
    type Unmapped = Self;
    /// Retrieve the handle of the shared memory
    /// # Arguments
    ///   * `label` - The label of the shared memory
    /// # Returns
    ///   * `Ok(u32)` - The handle of the shared memory
    /// # Example
    /// ```
    /// let handle = Shm::get_handle(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    fn get_handle(label: ShmLabel) -> Result<u32, Status> {
        match sentry_uapi::syscall::get_shm_handle(label) {
            Status::Ok => {}
            status => return Err(status),
        }

        let mut handle = 0_u32;
        match copy_from_kernel(&mut handle) {
            Ok(Status::Ok) => Ok(handle),
            Err(status) => Err(status),
            _ => Err(Status::Denied),
        }
    }
    /// Create a new shared memory
    /// # Arguments
    ///   * `label` - The label of the shared memory
    /// # Returns
    ///   * `Ok(Shm::Unmapped)` - The unmapped shared memory object
    /// # Example
    /// ```
    /// let shm = Shm::new(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    fn new(label: ShmLabel) -> Result<Self::Unmapped, Status> {
        let handle = Self::get_handle(label)?;

        Ok(Shm {
            handle: Some(handle),
            label: Some(label),
            mapped_to: None,
            perms: None,
            _marker: PhantomData,
        })
    }

    /// Retrieve info about the shared memory
    /// # Returns
    /// * `Ok(ShmInfo)` - The info of the shared memory
    /// # Example
    /// ```
    /// let info = shm.get_info();
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    fn get_info(&mut self) -> Result<ShmInfo, Status> {
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
                let new_handle = Self::get_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        sentry_uapi::syscall::shm_get_infos(handle);
        match copy_from_kernel(&mut shm_info) {
            Ok(Status::Ok) => Ok(shm_info),
            Err(status) => Err(status),
            _ => Err(Status::Denied),
        }
    }

    /// Set the credentials of the shared memory
    /// # Arguments
    /// * `to_task` - The task to set the credentials to
    /// * `perms` - The permissions to set
    /// # Returns
    /// * `Ok(Status::Ok)` - If the setting is done
    /// # Example
    /// ```
    /// let handle = shm.set_creds(0xAA, SHMPermission::Read | SHMPermission::Write);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn set_creds(&mut self, to_task: u32, perms: u32) -> Result<Status, Status> {
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
                let new_handle = Self::get_handle(label)?;
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
            status => Err(status),
        }
    }
    /// Get the credentials of the shared memory
    /// # Returns
    /// * `Ok(u32)` - The credentials of the shared memory
    /// # Example
    /// ```
    /// let creds = shm.get_creds();
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn get_creds(&mut self) -> Result<u32, Status> {
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
    fn is_mappable(&mut self) -> bool {
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
    fn is_readable(&mut self) -> bool {
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
    fn is_writable(&mut self) -> bool {
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
    fn is_transferable(&mut self) -> bool {
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
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn get_base(&mut self) -> Result<usize, Status> {
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
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn get_len(&mut self) -> Result<usize, Status> {
        // Get len from get_info
        let shm_info = self.get_info()?;
        Ok(shm_info.len)
    }
}

impl Mappable for Shm<Unmapped> {
    type Mapped = Shm<Mapped>;

    type Unmapped = Self;
    /// Map the shared memory by retrieving the handle
    /// # Arguments
    /// * `to_task` - The task to map the shared memory to
    /// # Returns
    /// * `Ok(Status::Ok)` - If the the mapping is done
    /// # Example
    /// ```
    /// let mapped_shm = shm.map(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn map(&mut self, to_task: u32) -> Result<Self::Mapped, Status> {
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
                let new_handle = Self::get_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };

        // Map the shared memory
        match sentry_uapi::syscall::map_shm(handle) {
            Status::Ok => {
                self.mapped_to = Some(to_task);
                Ok(Shm {
                    handle: Some(handle),
                    label: self.label,
                    mapped_to: Some(to_task),
                    perms: self.perms,
                    _marker: PhantomData::<Mapped>,
                })
            }
            status => Err(status),
        }
    }
}

impl SentryShm for Shm<Mapped> {
    type Mapped = Self;
    type Unmapped = Shm<Unmapped>;
    /// Retrieve the handle of the shared memory
    /// # Arguments
    ///   * `label` - The label of the shared memory
    /// # Returns
    ///   * `Ok(u32)` - The handle of the shared memory
    /// # Example
    /// ```
    /// let handle = Shm::get_handle(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    fn get_handle(label: ShmLabel) -> Result<u32, Status> {
        match sentry_uapi::syscall::get_shm_handle(label) {
            Status::Ok => {}
            status => return Err(status),
        }

        let mut handle = 0_u32;
        match copy_from_kernel(&mut handle) {
            Ok(Status::Ok) => Ok(handle),
            Err(status) => Err(status),
            _ => Err(Status::Denied),
        }
    }
    /// Create a new shared memory
    /// # Arguments
    ///   * `label` - The label of the shared memory
    /// # Returns
    ///   * `Ok(Shm)` - The shared memory object
    /// # Example
    /// ```
    /// let shm = Shm::new(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    fn new(label: ShmLabel) -> Result<Self::Unmapped, Status> {
        let handle = Self::get_handle(label)?;

        Ok(Shm {
            handle: Some(handle),
            label: Some(label),
            mapped_to: None,
            perms: None,
            _marker: PhantomData,
        })
    }

    /// Retrieve info about the shared memory
    /// # Arguments
    /// * `label` - The label of the shared memory
    /// # Returns
    /// * `Ok(ShmInfo)` - The info of the shared memory
    /// # Example
    /// ```
    /// let info = shm.get_info(0xAA);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    fn get_info(&mut self) -> Result<ShmInfo, Status> {
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
                let new_handle = Self::get_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        sentry_uapi::syscall::shm_get_infos(handle);
        match copy_from_kernel(&mut shm_info) {
            Ok(Status::Ok) => Ok(shm_info),
            Err(status) => Err(status),
            _ => Err(Status::Denied),
        }
    }

    /// Set the credentials of the shared memory
    /// # Arguments
    /// * `to_task` - The task to set the credentials to
    /// * `perms` - The permissions to set
    /// # Returns
    /// * `Ok(Status::Ok)` - If the setting is done
    /// # Example
    /// ```
    /// let handle = shm.set_creds(0xAA, SHMPermission::Read | SHMPermission::Write);
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn set_creds(&mut self, to_task: u32, perms: u32) -> Result<Status, Status> {
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
                let new_handle = Self::get_handle(label)?;
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
            status => Err(status),
        }
    }
    /// Get the credentials of the shared memory
    /// # Returns
    /// * `Ok(u32)` - The credentials of the shared memory
    /// # Example
    /// ```
    /// let creds = shm.get_creds();
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn get_creds(&mut self) -> Result<u32, Status> {
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
    fn is_mappable(&mut self) -> bool {
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
    fn is_readable(&mut self) -> bool {
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
    fn is_writable(&mut self) -> bool {
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
    fn is_transferable(&mut self) -> bool {
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
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn get_base(&mut self) -> Result<usize, Status> {
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
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn get_len(&mut self) -> Result<usize, Status> {
        // Get len from get_info
        let shm_info = self.get_info()?;
        Ok(shm_info.len)
    }
}

impl Unmappable for Shm<Mapped> {
    type Mapped = Self;
    type Unmapped = Shm<Unmapped>;
    /// Unmap the shared memory
    /// # Returns
    /// * `Ok(Status::Ok)` - If the unmapping is done
    /// # Example
    /// ```
    /// let unmapped_shm = mapped_shm.unmap();
    /// ```
    /// # Errors
    /// * `Status::Denied` - The requested action is not allowed for caller
    /// * `Status::Invalid` - At least one parameter is not valid (not allowed or not found)
    /// * `Status::Busy` - The requested resource do now allow the current call for now
    /// * `Status::NoEntity` - The requested resource was not found
    /// * `Status::Critical` - Critical (mostly security-related) unexpected event
    /// * `Status::Timeout` - The requested action timed out
    /// * `Status::Again` - The requested resource is not here yet, come back later
    /// * `Status::Intr` - The call has been interrupted sooner than expected. Used for blocking calls
    /// * `Status::Deadlk` - The requested resource can’t be manipulated without generating a dead lock
    /// * `Status::AlreadyMapped` - The requested resource is already mapped
    fn unmap(&mut self) -> Result<Self::Unmapped, Status> {
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
                let new_handle = Self::get_handle(label)?;
                self.handle = Some(new_handle);
                new_handle
            }
        };
        match sentry_uapi::syscall::unmap_shm(handle) {
            Status::Ok => {
                self.mapped_to = None;
                self.perms = None;
                Ok(Shm {
                    handle: Some(handle),
                    label: self.label,
                    mapped_to: None,
                    perms: self.perms,
                    _marker: PhantomData::<Unmapped>,
                })
            }
            status => Err(status),
        }
    }
}
