# SPDX-FileCopyrightText: 2023 - 2024 Ledger SAS
# SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

project('libshield', 'c', 'cpp', 'rust',
    meson_version: '>=1.4.0',
    version: run_command('dunamai', 'from', 'git', '--style', 'semver', '--dirty', check: true).stdout().strip(),
    default_options: [
        'c_std=gnu11', 'cpp_std=gnu++20', 'rust_std=2021',
        'warning_level=3', 'optimization=s',
        'default_library=static', 'b_staticpic=false',
        'licensedir=share/info/libshield',
        'pkgconfig.relocatable=true',
        'config=configs/defconfig',
])

pkg = import('pkgconfig')
fs = import('fs')

kconfig_file = meson.current_source_dir() / 'Kconfig'
kconfig_proj = subproject('kconfig', default_options: ['kconfig=@0@'.format(kconfig_file)])
kconfig_h = kconfig_proj.get_variable('kconfig_h')
kconfig_data = kconfig_proj.get_variable('kconfig_data')

shield_external_deps = []
shield_data_dir = get_option('datadir') / 'shield'

with_picolibc_opt = get_option('with_picolibc')


shield_link_args = []
# Extra variable to add to the generated pkg-config file
shield_pkgconfig_extravar = [ 'shield_datadir=${datadir}/shield' ]
# Extra link args for internal deps usage
shield_extra_link_args_internal = []
# Extra link args for external deps usage
shield_extra_link_args_pkgconfig = []
shield_compile_args = []

# Only build uapi, if configured in Kconfig **and** while cross compiling
if meson.is_cross_build()

if kconfig_data.get('CONFIG_WITH_SENTRY', 0) == 1
# in case of fallback to subproject, pass default_options to subproject (i.e. sentry)
# As sentry wrap file declares as `uapi `provider, fallback arg is implicit
uapi_dep = dependency('uapi',
    default_options: ['with_kernel=false', 'with_idle=false'],
)

shield_external_deps += uapi_dep

# FIXME: may be forwarded by uapi dep (or cross file)
shield_link_args += [
    '-Wl,--no-undefined',
    '-z', 'noexecstack',
    '-nostdlib',
    '-nodefaultlibs',
    '-nostartfiles',
]

# the picolibc support is based on the fact that picolibc is
# provided by the cross-toolchain itself, which is, when using Camelot-SDK
# a part of the SDK components
if with_picolibc_opt
    shield_link_args += '-specs=picolibc.specs'
endif

# XXX:
# In case of static PIC build, the app is linked as executable elf and relocates
# at project integration time by outpost buildsystem.
# All apps are linked against the same linker script (TODO: check soc mem capacities).
# All PIC relative option are handled by libshield and forwarded through dependency to
# consumer.
#
# In case of nopic build, the app is linked as relocatable elf (`-r` link opt, a.k.a. partial link)
# and relink with a dedicated (with correct memory partionning) linked script at integration time.
# Shield package install a linkerscript template that is feed by outpost buildsystem.

if get_option('b_staticpic')
    shield_app_linkerscript = join_paths('lnk', 'linkerscript.ld')
    shield_link_args += [ '-Wl,--gc-sections', '-static-pie' ]
    shield_extra_link_args_internal += [
        '-T@0@'.format(join_paths(meson.current_source_dir(), shield_app_linkerscript))
    ]
    shield_extra_link_args_pkgconfig += [ '-T${linker_script}']

    shield_compile_args += [ '-mno-pic-data-is-text-relative', '-msingle-pic-base' ]
else
    # In noPIC use template linker script
    shield_app_linkerscript = join_paths('lnk', 'linkerscript.ld.in')
    shield_link_args += [ '-r', '--entry', '_start' ]
endif

install_data(shield_app_linkerscript, install_dir : shield_data_dir)
shield_pkgconfig_extravar += [ 'linker_script=@0@'.format('${shield_datadir}' / fs.name(shield_app_linkerscript)) ]

endif # w/ sentry
endif # cross build

# about libshield basics

ssmod = import('sourceset')
shield_clib_sourceset = ssmod.source_set()

add_project_arguments(shield_compile_args, language: 'c')
add_project_arguments('-include', kconfig_h.full_path(), language : ['c'])
shield_clib_sourceset.add(kconfig_h)

subdir('include')
subdir('src')

# shield dependency as internal meson dep (i.e. use as subproject)
shield_c_dep = declare_dependency(
    include_directories: [ shield_inc ],
    link_with: [ shield_clib ],
    dependencies: [ shield_external_deps ],
    link_args: [ shield_link_args, shield_extra_link_args_internal ],
    compile_args: [ shield_compile_args ],
)

# shield dependency as external pkg-config deps
pkg.generate(shield_clib,
    filebase: 'shield',
    libraries: [shield_link_args, shield_extra_link_args_pkgconfig],
    variables: shield_pkgconfig_extravar,
    extra_cflags: [ shield_compile_args ],
)

# unittest suite
with_tests_opt = get_option('with_tests')
if with_tests_opt
    subdir('tests')
endif

# about documentation building
with_doc_opt = get_option('with_doc')
if with_doc_opt
    subdir('doc')
endif

summary(
    {
        'picolibc': with_picolibc_opt,
        'tests': with_tests_opt,
        'doc': with_doc_opt,
    },
    bool_yn: true,
    section: 'Configuration',
)
