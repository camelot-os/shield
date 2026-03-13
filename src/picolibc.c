// SPDX-FileCopyrightText: 2023 - 2024 Ledger SAS
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause


#include <stddef.h>
#include <stdint.h>
#include <time.h>
#include <stddef.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/time.h>
#include <signal.h>

#include <shield/sentry.h>
#include <shield/private/coreutils.h>


extern uint32_t _sheap;
extern uint32_t _eheap;


/* symbol declaration, resolved using picolibc, associated implementation are only stubs by now */

#define PICOLIBC_WEAK __attribute__((weak))

/*@ */
__attribute__((used))
void _exit(int status)
{
    sys_exit(status);
    __builtin_unreachable();
}

/**
 * @brief Read from a file descriptor.
 *
 * This function only allows reading from stdin, which is, on sentry kernel,
 * mapped to the SVC exchange area. The function checks for valid parameters and then
 * copies data from the kernel into the provided buffer. It returns the number of bytes
 * read or -1 on error.
 *
 * @param fd File descriptor to read from (must be 0 for stdin).
 * @param buf Buffer to store the read data.
 * @param nbyte Number of bytes to read.
 * @return Number of bytes read, or -1 on error.
 */
__attribute__((used))
ssize_t read(int fd, void *buf, size_t nbyte)
{
    int res = -1;
    if (unlikely(fd != 0)) {
        goto end;
    }
    if (unlikely(buf == NULL || nbyte == 0U)) {
        goto end;
    }
    if (unlikely(svcexchange_get_maxlen() < nbyte || nbyte == 0U)) {
        goto end;
    }
    copy_from_kernel(buf, nbyte);
    res = (ssize_t)nbyte;
end:
    return res;
}

/**
 * @brief Write to a file descriptor.
 *
 * This function only allows writing to stdout and stderr, which are, on sentry kernel,
 * mapped to the sys_log syscall. The function checks for valid parameters and then emits a kernel log
 * with the provided data. It returns the number of bytes written or -1 on error.
 *
 * Note that sys_log() can be used in order to emit binary data over the configured serial port if
 * needed.
 * In a possible future implementation, a given user service that allows dedicated fd for writing
 * data (e.g. using a filesystem service) can be implemented, and thus will need to override this
 * write() implementation.
 *
 * @see sys_log() for more details on the logging mechanism and limitations.
 * @param fd File descriptor to write to (must be 1 for stdout or 2 for stderr).
 * @param buf Buffer containing the data to write.
 * @param nbyte Number of bytes to write.
 * @return Number of bytes written, or -1 on error.
 */
__attribute__((used))
ssize_t write(int fd, const void *buf, size_t nbyte)
{
    int res = -1;
    if (unlikely(fd != 1 || fd != 2)) {
        goto end;
    }
    /* ensure that no modulo overflow occurs */
    if (unlikely(nbyte > SIZE_MAX)) {
        goto end;
    }
    if (unlikely(buf == NULL || nbyte == 0U)) {
        goto end;
    }
    if (unlikely(sys_log(buf, nbyte) != STATUS_OK)) {
        goto end;
    }
    res = (ssize_t)nbyte;
end:
    return res;
}

/**
 * @brief Fill a buffer with random bytes.
 *
 * This function fills the provided buffer with random bytes by repeatedly calling `sys_get_random()`
 * until the requested number of bytes is filled. It checks for valid parameters and returns 0
 * on success or -1 on error.
 *
 * @param buffer Buffer to fill with random bytes.
 * @param length Number of bytes to fill.
 * @return 0 on success, -1 on error.
 */
__attribute__((used))
int getentropy(void *buffer, size_t length)
{
    int res = -1;
    uint8_t *out = (uint8_t *)buffer;
    uint32_t random = 0;
    size_t offset = 0;

    if (unlikely(buffer == NULL)) {
        goto end;
    }

    while (offset < length) {
        size_t chunk = length - offset;

        if (unlikely(sys_get_random(&random) != STATUS_OK)) {
            goto end;
        }

        if (chunk > sizeof(random)) {
            chunk = sizeof(random);
        }

        for (size_t i = 0; i < chunk; ++i) {
            out[offset + i] = (uint8_t)(random >> (8U * i));
        }

        offset += chunk;
    }
    res = 0;
end:
    return res;
}

/**
 * @brief Close a file descriptor.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at uper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param fd File descriptor to close.
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK int close(int fd)
{
    (void)fd;
    return -1;
}

/**
 * @brief Get file status.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param fd File descriptor to get status for.
 * @param statbuf Buffer to store the file status.
 * @return -1 to indicate failure.
 */
PICOLIBC_WEAK int fstat(int fd, struct stat *statbuf)
{
    (void)fd;
    (void)statbuf;
    return -1;
}

/**
 * @brief Get the current time of day.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * Having a working implementation requires an existing RTC device and a user service
 * that allows to get the current time from it, which is not available in this default implementation.
 *
 * @param tv Buffer to store the current time of day.
 * @param tz Buffer to store the timezone information (not used in this implementation).
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK int gettimeofday(struct timeval *tv, void *tz)
{
    (void)tv;
    (void)tz;
    return -1;
}

/**
 * @brief Move the file offset.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param fd File descriptor to move the offset for.
 * @param offset Offset to move to.
 * @param whence Position from which offset is applied.
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK off_t lseek(int fd, off_t offset, int whence)
{
    (void)fd;
    (void)offset;
    (void)whence;
    return (off_t)-1;
}

/**
 * @brief Open a file.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param path Path to the file to open.
 * @param flags Flags for opening the file.
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK int open(char *path, int flags, ...)
{
    (void)path;
    (void)flags;
    return -1;
}

/**
 * @brief Change the signal mask.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param how How to change the signal mask.
 * @param set New signal mask.
 * @param oldset Buffer to store the old signal mask.
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK int sigprocmask(int how, const sigset_t *set, sigset_t *oldset)
{
    (void)how;
    (void)set;
    (void)oldset;
    return -1;
}

/**
 * @brief Get file status by path.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param path Path to the file to get status for.
 * @param statbuf Buffer to store the file status.
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK int stat(const char *path, struct stat *statbuf)
{
    (void)path;
    (void)statbuf;
    return -1;
}

/**
 * @brief Unlink a file.
 *
 * This function is a weak symbol and will return -1 by default. It can be
 * overridden by providing an implementation that matches the signature at upper
 * levels of the application. The default implementation does not perform any
 * operations and simply returns -1 to indicate failure.
 *
 * @param path Path to the file to unlink.
 * @return -1 to indicate failure.
 */
__attribute__((used))
PICOLIBC_WEAK int unlink(char *path)
{
    (void)path;
    return -1;
}
