# SPDX-FileCopyrightText: 2023 - 2024 Ledger SAS
#
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

# for private headers, only compile time
subdir('include')

if not with_picolibc_opt
# when using picolibc, no need for local ulibc implementation,
# as picolibc will provide the libc symbols
shield_clib_sourceset.add(files(
    'abs.c',
    'assert.c',
    'errno.c',
    'string.c',
    'arpa/inet.c',
    'pthread.c',
    'time.c',
    'printf.c',
    'printf_lexer.c',
    'rand.c',
    'signal.c',
    'sys/msg.c',
))
else
shield_clib_sourceset.add(files(
    'picolibc.c',
))
endif

# files that are always required
shield_clib_sourceset.add(
    when: 'CONFIG_WITH_SENTRY',
    if_true: files(
        'support/sentry.c',
        'support/sentry.h',
        'entrypoint/thread_init.c',
        'entrypoint/libc_init.c')
)

# applying config, from local or parent
shield_clib_sourceset_config = shield_clib_sourceset.apply(kconfig_data, strict: false)

shield_clib = static_library(
    'shield_c_lib',
    sources : [ shield_clib_sourceset_config.sources(), shield_private_headers ],
    include_directories: [ shield_inc, shield_private_inc ],
    dependencies: [ shield_external_deps ],
    install: true,
)
