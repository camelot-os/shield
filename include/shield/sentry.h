// Copyright (c) 2026 H2Lab Development Team
// SPDX-License-Identifier: Apache-2.0 or BSD-3-Clause

#ifndef SHIELD_SENTRY_H
#define SHIELD_SENTRY_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <limits.h>

#include <uapi.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Schedule an alarm signal for the current task.
 *
 * @param timeout_ms Delay in milliseconds before signal delivery.
 * @return Syscall status.
 */
inline Status sys_alarm(uint32_t timeout_ms)
{
	return __sys_alarm(timeout_ms);
}

/**
 * @brief Terminate the current process.
 *
 * @param status Process exit status code.
 * @return Syscall status.
 */
inline Status sys_exit(int32_t status)
{
	return __sys_exit(status);
}

/**
 * @brief Retrieve a time/cycle counter value from the kernel.
 *
 * This API calls `__sys_get_cycle()` and then copies the result from
 * the SVC exchange area.
 *
 * @param precision Requested precision.
 * @param cycle Output value (u64).
 * @return Syscall or kernel-copy status.
 */
inline Status sys_get_cycle(Precision precision, uint64_t *cycle)
{
	Status status = STATUS_INVALID;

	if (cycle == NULL) {
		goto end;
	}

	status = __sys_get_cycle(precision);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)cycle, sizeof(*cycle));
end:
    return status;
}

/**
 * @brief Retrieve a device handle from a DTS label.
 *
 * This API calls `__sys_get_device_handle()` and then copies the handle
 * from the SVC exchange area.
 *
 * @param devlabel Device label.
 * @param dev Output handle.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_get_device_handle(uint8_t devlabel, devh_t *dev)
{
	Status status = STATUS_INVALID;

	if (dev == NULL) {
		goto end;
	}

	status = __sys_get_device_handle(devlabel);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)dev, sizeof(*dev));
end:
	return status;
}

/**
 * @brief Retrieve a DMA stream handle from a DTS label.
 *
 * This API calls `__sys_get_dma_stream_handle()` and then copies the handle
 * from the SVC exchange area.
 *
 * @param label DMA stream label.
 * @param stream Output handle.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_get_dma_stream_handle(uint32_t label, dmah_t *stream)
{
	Status status = STATUS_INVALID;

	if (stream == NULL) {
		goto end;
	}

	status = __sys_get_dma_stream_handle(label);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)stream, sizeof(*stream));
end:
	return status;
}

/**
 * @brief Retrieve an SHM handle from an SHM label.
 *
 * This API calls `__sys_get_shm_handle()` and then copies the handle
 * from the SVC exchange area.
 *
 * @param shmlabel SHM label.
 * @param shm Output handle.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_get_shm_handle(uint32_t shmlabel, shmh_t *shm)
{
	Status status = STATUS_INVALID;

	if (shm == NULL) {
		goto end;
	}

	status = __sys_get_shm_handle(shmlabel);
	if (status != STATUS_OK) {
		goto end;
	}
    status = copy_from_kernel((uint8_t *)shm, sizeof(*shm));
end:
	return status;
}

/**
 * @brief Retrieve a process handle from a process label.
 *
 * This API calls `__sys_get_process_handle()` and then copies the handle
 * from the SVC exchange area.
 *
 * @param process Target process label.
 * @param handle Output handle.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_get_process_handle(ProcessLabel process, taskh_t *handle)
{
	Status status = STATUS_INVALID;

	if (handle == NULL) {
		goto end;
	}

	status = __sys_get_process_handle(process);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)handle, sizeof(*handle));
end:
	return status;
}

/**
 * @brief Retrieve a 32-bit random value from the kernel.
 *
 * This API calls `__sys_get_random()` and then copies the value from
 * the SVC exchange area.
 *
 * @param random Output random value.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_get_random(uint32_t *random)
{
	Status status = STATUS_INVALID;

	if (random == NULL) {
		goto end;
	}

	status = __sys_get_random();
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)random, sizeof(*random));
end:
	return status;
}

/**
 * @brief Configure a GPIO associated with a device resource.
 *
 * @param resource Device resource handle.
 * @param io GPIO index.
 * @return Syscall status.
 */
inline Status sys_gpio_configure(uint32_t resource, uint8_t io)
{
	return __sys_gpio_configure(resource, io);
}

/**
 * @brief Read a GPIO state associated with a device resource.
 *
 * This API calls `__sys_gpio_get()` and then copies the GPIO value from
 * the SVC exchange area.
 *
 * @param resource Device resource handle.
 * @param io GPIO index.
 * @param val Output value.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_gpio_get(uint32_t resource, uint8_t io, bool *val)
{
	Status status = STATUS_INVALID;
	uint8_t value;

	if (val == NULL) {
		goto end;
	}

	status = __sys_gpio_get(resource, io);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel(&value, sizeof(value));
	if (status != STATUS_OK) {
		goto end;
	}

	*val = (value != 0U);
	status = STATUS_OK;
end:
	return status;
}

/**
 * @brief Drive a GPIO low.
 *
 * @param resource Device resource handle.
 * @param io GPIO index.
 * @return Syscall status.
 */
inline Status sys_gpio_reset(uint32_t resource, uint8_t io)
{
	return __sys_gpio_reset(resource, io);
}

/**
 * @brief Set a GPIO to the requested value.
 *
 * @param resource Device resource handle.
 * @param io GPIO index.
 * @param val GPIO value to write.
 * @return Syscall status.
 */
inline Status sys_gpio_set(uint32_t resource, uint8_t io, bool val)
{
	return __sys_gpio_set(resource, io, val);
}

/**
 * @brief Toggle a GPIO state.
 *
 * @param resource Device resource handle.
 * @param io GPIO index.
 * @return Syscall status.
 */
inline Status sys_gpio_toggle(uint32_t resource, uint8_t io)
{
	return __sys_gpio_toggle(resource, io);
}

/**
 * @brief Acknowledge an interrupt at controller level.
 *
 * @param irq IRQ number.
 * @return Syscall status.
 */
inline Status sys_irq_acknowledge(uint16_t irq)
{
	return __sys_irq_acknowledge(irq);
}

/**
 * @brief Enable an interrupt at controller level.
 *
 * @param irq IRQ number.
 * @return Syscall status.
 */
inline Status sys_irq_enable(uint16_t irq)
{
	return __sys_irq_enable(irq);
}

/**
 * @brief Disable an interrupt at controller level.
 *
 * @param irq IRQ number.
 * @return Syscall status.
 */
inline Status sys_irq_disable(uint16_t irq)
{
	return __sys_irq_disable(irq);
}

/**
 * @brief Emit a kernel log after copying the payload into SVC exchange.
 *
 * @param data Buffer to log.
 * @param length Buffer length.
 * @return Copy or syscall status.
 */
inline Status sys_log(const uint8_t *data, size_t length)
{
	Status status = STATUS_INVALID;
	size_t maxlen = svcexchange_get_maxlen();
	size_t offset = 0U;

	if ((data == NULL) && (length != 0U)) {
		goto end;
	}
	if ((length != 0U) && (maxlen == 0U)) {
		goto end;
	}
	if (length == 0U) {
		status = __sys_log(0U);
		goto end;
	}

	while (offset < length) {
		size_t chunk_len = length - offset;

		if (chunk_len > maxlen) {
			chunk_len = maxlen;
		}

		status = copy_to_kernel(&data[offset], chunk_len);
		if (status != STATUS_OK) {
			goto end;
		}

		status = __sys_log(chunk_len);
		if (status != STATUS_OK) {
			goto end;
		}

		offset += chunk_len;
	}
end:
	return status;
}

/**
 * @brief Configure CPU sleep behavior.
 *
 * @param mode CPU sleep mode.
 * @return Syscall status.
 */
inline Status sys_manage_cpu_sleep(CPUSleep mode)
{
	return __sys_manage_cpu_sleep(mode);
}

/**
 * @brief Map a device resource.
 *
 * @param dev Handle device.
 * @return Syscall status.
 */
inline Status sys_map_dev(devh_t dev)
{
	return __sys_map_dev(dev);
}

/**
 * @brief Map an SHM resource.
 *
 * @param shm Handle SHM.
 * @return Syscall status.
 */
inline Status sys_map_shm(shmh_t shm)
{
	return __sys_map_shm(shm);
}

/**
 * @brief Send an IPC message.
 *
 * This API first copies the message into the SVC exchange area, then calls
 * `__sys_send_ipc()`.
 *
 * @param resource Target task handle.
 * @param data Message buffer.
 * @param length Message length.
 * @return Copy or syscall status.
 */
inline Status sys_send_ipc(uint32_t resource, const uint8_t *data, size_t length)
{
	Status status = STATUS_INVALID;

	if ((data == NULL) && (length != 0U)) {
		goto end;
	}
	if ((length > svcexchange_get_maxlen()) || (length > UINT8_MAX)) {
		goto end;
	}

	if (length != 0U) {
		status = copy_to_kernel(data, length);
		if (status != STATUS_OK) {
			goto end;
		}
	}

	status = __sys_send_ipc(resource, (uint8_t)length);
end:
	return status;
}

/**
 * @brief Send a signal to another process.
 *
 * @param resource Target task handle.
 * @param signal_type Signal to send.
 * @return Syscall status.
 */
inline Status sys_send_signal(uint32_t resource, Signal signal_type)
{
	return __sys_send_signal(resource, signal_type);
}

/**
 * @brief Set SHM permissions for a target task.
 *
 * @param shm Handle SHM.
 * @param target Target task handle.
 * @param shm_perm SHM permission mask.
 * @return Syscall status.
 */
inline Status sys_shm_set_credential(shmh_t shm, taskh_t target, uint32_t shm_perm)
{
	return __sys_shm_set_credential(shm, target, shm_perm);
}

/**
 * @brief Retrieve SHM information into a user structure.
 *
 * This API calls `__sys_shm_get_infos()` and then copies a
 * `shm_infos_t` structure from the SVC exchange area.
 *
 * @param shm Handle SHM.
 * @param infos Output structure.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_shm_get_infos(shmh_t shm, shm_infos_t *infos)
{
	Status status = STATUS_INVALID;

	if (infos == NULL) {
		goto end;
	}

	status = __sys_shm_get_infos(shm);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)infos, sizeof(*infos));
end:
	return status;
}

/**
 * @brief Put the task to sleep.
 *
 * @param duration_ms Sleep duration.
 * @param mode Sleep mode.
 * @return Syscall status.
 */
inline Status sys_sleep(SleepDuration duration_ms, SleepMode mode)
{
	return __sys_sleep(duration_ms, mode);
}

/**
 * @brief Start a process from its label.
 *
 * @param process Target process label.
 * @return Syscall status.
 */
inline Status sys_start(ProcessLabel process)
{
	return __sys_start(process);
}

/**
 * @brief Unmap a device resource.
 *
 * @param dev Handle device.
 * @return Syscall status.
 */
inline Status sys_unmap_dev(devh_t dev)
{
	return __sys_unmap_dev(dev);
}

/**
 * @brief Unmap an SHM resource.
 *
 * @param shm Handle SHM.
 * @return Syscall status.
 */
inline Status sys_unmap_shm(shmh_t shm)
{
	return __sys_unmap_shm(shm);
}

/**
 * @brief Wait for an event and optionally copy event data.
 *
 * This API calls `__sys_wait_for_event()`. On success, if `event` is non-null,
 * the function copies `event_len` bytes from the SVC exchange area.
 *
 * @param mask Mask of expected events.
 * @param timeout Associated timeout.
 * @param event Optional output event buffer.
 * @param event_len Number of bytes to copy into `event`.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_wait_for_event(uint8_t mask, int32_t timeout, void *event, size_t event_len)
{
	Status status = STATUS_INVALID;

	if ((event == NULL) && (event_len != 0U)) {
		goto end;
	}
	if (event_len > svcexchange_get_maxlen()) {
		goto end;
	}

	status = __sys_wait_for_event(mask, timeout);
	if (status != STATUS_OK) {
		goto end;
	}

	if ((event != NULL) && (event_len != 0U)) {
		status = copy_from_kernel((uint8_t *)event, event_len);
	}

end:
	return status;
}

/**
 * @brief Voluntarily yield the CPU.
 *
 * @return Syscall status.
 */
inline Status sys_sched_yield(void)
{
	return __sys_sched_yield();
}

/**
 * @brief Update a clock register through PM.
 *
 * @param clk_reg Target register.
 * @param regmsk Bit mask.
 * @param val Value to apply.
 * @return Syscall status.
 */
inline Status sys_pm_set_clock(uint32_t clk_reg, uint32_t regmsk, uint32_t val)
{
	return __sys_pm_set_clock(clk_reg, regmsk, val);
}

/**
 * @brief Start a DMA stream.
 *
 * @param stream Handle DMA.
 * @return Syscall status.
 */
inline Status sys_dma_start_stream(dmah_t stream)
{
	return __sys_dma_start_stream(stream);
}

/**
 * @brief Suspend a DMA stream.
 *
 * @param stream Handle DMA.
 * @return Syscall status.
 */
inline Status sys_dma_suspend_stream(dmah_t stream)
{
	return __sys_dma_suspend_stream(stream);
}

/**
 * @brief Resume a suspended DMA stream.
 *
 * @param stream Handle DMA.
 * @return Syscall status.
 */
inline Status sys_dma_resume_stream(dmah_t stream)
{
	return __sys_dma_resume_stream(stream);
}

/**
 * @brief Assign a DMA stream to its hardware channel.
 *
 * @param stream Handle DMA.
 * @return Syscall status.
 */
inline Status sys_dma_assign_stream(dmah_t stream)
{
	return __sys_dma_assign_stream(stream);
}

/**
 * @brief Unassign a DMA stream from its hardware channel.
 *
 * @param stream Handle DMA.
 * @return Syscall status.
 */
inline Status sys_dma_unassign_stream(dmah_t stream)
{
	return __sys_dma_unassign_stream(stream);
}

/**
 * @brief Retrieve the status of a DMA stream.
 *
 * This API calls `__sys_dma_get_stream_status()` and then copies one status
 * byte from the SVC exchange area.
 *
 * @param stream Handle DMA.
 * @param stream_status Output status byte.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_dma_get_stream_status(dmah_t stream, uint8_t *stream_status)
{
	Status status = STATUS_INVALID;

	if (stream_status == NULL) {
		goto end;
	}

	status = __sys_dma_get_stream_status(stream);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel(stream_status, sizeof(*stream_status));

end:
	return status;
}

/**
 * @brief Retrieve information for a DMA stream.
 *
 * This API calls `__sys_dma_get_stream_info()` and then copies `info_len`
 * bytes from the SVC exchange area into `stream_info`.
 *
 * @param stream Handle DMA.
 * @param stream_info Output buffer.
 * @param info_len Number of bytes to copy.
 * @return Syscall or kernel-copy status.
 */
inline Status sys_dma_get_stream_info(dmah_t stream, void *stream_info, size_t info_len)
{
	Status status = STATUS_INVALID;

	if ((stream_info == NULL) || (info_len == 0U)) {
		goto end;
	}
	if (info_len > svcexchange_get_maxlen()) {
		goto end;
	}

	status = __sys_dma_get_stream_info(stream);
	if (status != STATUS_OK) {
		goto end;
	}

	status = copy_from_kernel((uint8_t *)stream_info, info_len);

end:
	return status;
}

#ifdef __cplusplus
}
#endif


#endif/*!SHIELD_SENTRY_H*/
