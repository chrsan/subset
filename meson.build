project('subset', 'c', 'cpp')

harfbuzz_proj = subproject(
  'harfbuzz',
  default_options: ['default_library=static'],
)

harfbuzz_dep = harfbuzz_proj.get_variable('harfbuzz_dep')

sheenbidi_proj = subproject(
  'sheenbidi',
  default_options: ['default_library=static'],
)

sheenbidi_dep = sheenbidi_proj.get_variable('sheenbidi_dep')

subset_inc = include_directories('src')
subset_sources = files('src/subset.cc')
subset_lib = library(
  'subset',
  subset_sources,
  cpp_args: ['-std=c++17', '-fno-exceptions', '-fno-rtti'],
  dependencies: [harfbuzz_dep, sheenbidi_dep],
  include_directories: subset_inc,
)

subset_dep = declare_dependency(
  include_directories: subset_inc,
  link_with: subset_lib,
)
