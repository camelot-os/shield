// SPDX-FileCopyrightText: 2024 Ledger SAS
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

unsafe extern "Rust" {
    /// External no_mangled, Rust ABI, `main` function declaration
    /// symbol defined in main.rs of the binary crate w/ [`shield-startup-macros::shield_main`] macro
    fn main();
}

///  Canari variable, as defined in LLVM & GCC compiler documentation, in order to
///  be manipulated each time a new frame is added on stack
#[unsafe(no_mangle)]
#[used]
static mut __stack_chk_guard: u32 = 0;

/// Outpost Thread entrypoint
///
/// When starting a thread, the thread identifier and the SSP seed is
/// passed by the Sentry kernel as arguments.
///
/// The seed is used to set the compiler-handled SSP value.
#[unsafe(no_mangle)]
pub extern "C" fn _start(_thread_id: u32, seed: u32) -> ! {
    unsafe {
        __stack_chk_guard = seed;
    }

    // TODO init seed for rand_r ?
    // rustlang initialisation ? heap for custom allocator ?

    // XXX: as main is extern, call is unsafe by construction.
    unsafe {
        main();
    }

    // TODO call uapi::sys::exit();

    panic!();
}
