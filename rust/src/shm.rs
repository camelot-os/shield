// SPDX-FileCopyrightText: 2025 Stephane N (ANSSI)
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::pedantic)]

use core::marker::PhantomData;
use sentry_uapi::copy_from_kernel;
use sentry_uapi::systypes::SHMPermission;
use uapi::systypes::shm::ShmInfo;
use uapi::systypes::{ShmHandle, ShmLabel, Status};

/// Marker type representing an **unmapped** shared memory.
pub struct Unmapped;

/// Marker type representing a **mapped** shared memory.
pub struct Mapped;

/// Shared Memory abstraction using the *typestate* pattern.
///
/// The state of the shared memory (mapped or unmapped) is encoded in the type
/// system, preventing invalid operations at compile time.
///
/// # Typestate
/// - [`Shm<Unmapped>`]: shared memory exists but is not mapped
/// - [`Shm<Mapped>`]: shared memory is mapped
///
/// # Invariants
/// - A `Shm` always owns a valid kernel handle
/// - Mapping / unmapping transitions are type-safe
pub struct Shm<State> {
    handle: ShmHandle,
    label: ShmLabel,
    info_cache: Option<ShmInfo>,
    _state: PhantomData<State>,
}

impl<State> Shm<State> {
    /* --------------------------------------------------------------------- */
    /* Internal helpers                                                       */
    /* --------------------------------------------------------------------- */

    /// Retrieve a shared memory handle from a label.
    ///
    /// This performs a syscall followed by a copy from kernel space.
    fn fetch_handle(label: ShmLabel) -> Result<ShmHandle, Status> {
        match sentry_uapi::syscall::get_shm_handle(label) {
            Status::Ok => {}
            status => return Err(status),
        }

        let mut handle = 0;
        match copy_from_kernel(&mut handle) {
            Ok(Status::Ok) => Ok(handle),
            Ok(status) | Err(status) => Err(status),
        }
    }

    /// Refresh cached shared memory information from the kernel.
    fn refresh_info(&mut self) -> Result<&ShmInfo, Status> {
        let mut info = ShmInfo {
            label: 0,
            handle: 0,
            base: 0,
            len: 0,
            perms: 0,
        };

        sentry_uapi::syscall::shm_get_infos(self.handle);
        match copy_from_kernel(&mut info) {
            Ok(Status::Ok) => {
                self.info_cache = Some(info);
                // SAFETY: just inserted
                match self.info_cache {
                    Some(ref info) => Ok(info),
                    None => Err(Status::Critical),
                }
            }
            Ok(status) | Err(status) => Err(status),
        }
    }

    /// Return cached information or refresh it if needed.
    fn info(&mut self) -> Result<&ShmInfo, Status> {
        if let Some(ref info) = self.info_cache {
            return Ok(info);
        }

        self.refresh_info()
    }

    fn has_permission(&mut self, perm: SHMPermission) -> bool {
        self.info()
            .map(|info| info.perms & perm as u32 != 0)
            .unwrap_or(false)
    }

    /* --------------------------------------------------------------------- */
    /* Public accessors                                                       */
    /* --------------------------------------------------------------------- */

    /// Return the permission mask of the shared memory.
    ///
    /// # Errors
    /// Propagates kernel errors if information retrieval fails.
    pub fn permissions(&mut self) -> Result<u32, Status> {
        Ok(self.info()?.perms)
    }

    /// Return the base virtual address of the shared memory.
    ///
    /// # Errors
    /// Propagates kernel errors if information retrieval fails.
    pub fn base_address(&mut self) -> Result<usize, Status> {
        Ok(self.info()?.base)
    }

    /// Return the length (in bytes) of the shared memory.
    ///
    /// # Errors
    /// Propagates kernel errors if information retrieval fails.
    pub fn length(&mut self) -> Result<usize, Status> {
        Ok(self.info()?.len)
    }

    /// Check whether the shared memory is readable.
    pub fn is_readable(&mut self) -> bool {
        self.has_permission(SHMPermission::Read)
    }

    /// Check whether the shared memory is writable.
    pub fn is_writable(&mut self) -> bool {
        self.has_permission(SHMPermission::Write)
    }

    /// Check whether the shared memory can be transferred to another task.
    pub fn is_transferable(&mut self) -> bool {
        self.has_permission(SHMPermission::Transfer)
    }

    /// Check whether the shared memory can be mapped.
    pub fn is_mappable(&mut self) -> bool {
        self.has_permission(SHMPermission::Map)
    }
}

/* ------------------------------------------------------------------------- */
/* Unmapped state                                                             */
/* ------------------------------------------------------------------------- */

impl Shm<Unmapped> {
    /// Create a new shared memory object in the **unmapped** state.
    ///
    /// This does **not** map the memory; it only retrieves a handle.
    ///
    /// # Arguments
    /// * `label` - Kernel shared memory label
    ///
    /// # Errors
    /// Returns any kernel error encountered during handle retrieval.
    pub fn new(label: ShmLabel) -> Result<Self, Status> {
        let handle = Self::fetch_handle(label)?;

        Ok(Self {
            handle,
            label,
            info_cache: None,
            _state: PhantomData,
        })
    }

    /// Map the shared memory into the current address space.
    ///
    /// On success, this consumes `self` and returns a [`Shm<Mapped>`].
    ///
    /// # Arguments
    /// * `to_task` - Target task identifier
    ///
    /// # Errors
    /// Returns kernel errors such as:
    /// - `Status::Denied`
    /// - `Status::Busy`
    /// - `Status::Invalid`
    pub fn map(self, _to_task: u32) -> Result<Shm<Mapped>, Status> {
        match sentry_uapi::syscall::map_shm(self.handle) {
            Status::Ok => Ok(Shm {
                handle: self.handle,
                label: self.label,
                info_cache: None,
                _state: PhantomData,
            }),
            status => Err(status),
        }
    }

    /// Set access permissions for another task.
    ///
    /// This operation is only valid while the memory is **unmapped**.
    ///
    /// # Arguments
    /// * `to_task` - Target task identifier
    /// * `perms` - Permission bitmask
    ///
    /// # Errors
    /// Returns kernel errors if permission update fails.
    pub fn set_credentials(&mut self, to_task: u32, perms: u32) -> Result<(), Status> {
        match sentry_uapi::syscall::shm_set_credential(self.handle, to_task, perms) {
            Status::Ok => {
                self.info_cache = None;
                Ok(())
            }
            status => Err(status),
        }
    }
}

/* ------------------------------------------------------------------------- */
/* Mapped state                                                               */
/* ------------------------------------------------------------------------- */

impl Shm<Mapped> {
    /// Unmap the shared memory.
    ///
    /// This consumes `self` and returns a [`Shm<Unmapped>`].
    ///
    /// # Errors
    /// Returns kernel errors if unmapping fails.
    pub fn unmap(self) -> Result<Shm<Unmapped>, Status> {
        match sentry_uapi::syscall::unmap_shm(self.handle) {
            Status::Ok => Ok(Shm {
                handle: self.handle,
                label: self.label,
                info_cache: None,
                _state: PhantomData,
            }),
            status => Err(status),
        }
    }
}
