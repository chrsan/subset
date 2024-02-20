subset_lib_inc = include_directories('.')
subset_lib = static_library('subset', 'subset.cc',
  cpp_args: ['-std=c++17', '-fno-exceptions', '-fno-rtti'],
  dependencies: [harfbuzz_dep, sheenbidi_dep],
  include_directories: subset_lib_inc,
)

subset_lib_dep = declare_dependency(
  include_directories: subset_lib_inc,
  link_with: subset_lib,
)
