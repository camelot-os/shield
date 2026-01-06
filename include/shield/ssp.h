// SPDX-FileCopyrightText: 2026 H2Lab Development Team
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

#ifndef SSP_H
#define SSP_H

#include <types.h>

#ifdef __ARM__

static inline void __stack_chk_fail(void)
{
    /* Inform Kernel that a SSP check has failed trough exit code */
    __asm__ volatile(
        "ldr r0, =123UL\n" /* exit code in r0 */
        "svc %0\n"
        :: "I" SYSCALL_EXIT
    );
    /* never reached */
}
#endif

#endif /* SSP_H */
