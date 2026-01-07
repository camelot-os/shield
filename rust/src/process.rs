// SPDX-FileCopyrightText: 2025 Stephane N (ANSSI)
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use sentry_uapi::copy_from_kernel;
use uapi::systypes::Status;
use uapi::systypes::TaskLabel;

/// This function retrieves the process handle associated with a given
/// task label.
/// It uses the `sentry_uapi::syscall::get_process_handle` syscall to
/// request the kernel to provide the handle.
/// If the syscall is successful, it copies the handle from the kernel
/// memory to the user space using `copy_from_kernel`.
/// If the syscall fails or if the handle cannot be copied, it returns
/// an error status.
/// # Arguments
/// * `task` - A `TaskLabel` representing the task for which we want
///   to retrieve the process handle.
/// # Returns
/// * `Ok(u32)` - The process handle associated with the task.
/// * `Err(Status)` - An error status indicating the failure reason.
/// # Example
/// ```
/// let task_label = 0x12 as u32;
/// match get_process_handle(task_label) {
///     Ok(handle) => println!("Process handle: {}", handle),
///     Err(status) => println!("Failed to get process handle: {:?}", status),
/// }
/// ```
/// # Errors
///
/// Will return `Err` if denied by the sentry kernel.
pub fn get_process_handle(task: TaskLabel) -> Result<u32, Status> {
    if sentry_uapi::syscall::get_process_handle(task) != Status::Ok {
        return Err(Status::Denied);
    }

    let mut handle = 0_u32;
    match copy_from_kernel(&mut handle) {
        Ok(Status::Ok) => Ok(handle),
        _ => Err(Status::Denied),
    }
}
