harbuzz_public_cpp_args = ['-DHB_NO_FALLBACK_SHAPE', '-DHB_NO_WIN1256']
harfbuzz_inc = include_directories('harfbuzz/src')
harfbuzz_lib = static_library('harfbuzz', 'harfbuzz/src/harfbuzz.cc',
  cpp_args: harbuzz_public_cpp_args + [
    '-std=c++11',
    '-fno-exceptions',
    '-fno-rtti',
    '-fno-threadsafe-statics',
  ],
  include_directories: harfbuzz_inc,
)

harfbuzz_dep = declare_dependency(
  compile_args: harbuzz_public_cpp_args,
  include_directories: harfbuzz_inc,
  link_with: harfbuzz_lib,
)

sheenbidi_public_c_args = ['-DSB_CONFIG_UNITY']
sheenbidi_inc = include_directories('SheenBidi/Headers')
sheenbidi_lib = static_library('sheenbidi', 'SheenBidi/Source/SheenBidi.c',
  c_args: sheenbidi_public_c_args,
  include_directories: sheenbidi_inc,
)

sheenbidi_dep = declare_dependency(
  compile_args: sheenbidi_public_c_args,
  include_directories: sheenbidi_inc,
  link_with: sheenbidi_lib,
)
