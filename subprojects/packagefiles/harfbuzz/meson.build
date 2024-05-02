project('harfbuzz', 'c', 'cpp')

harbuzz_public_cpp_args = ['-DHB_NO_FALLBACK_SHAPE', '-DHB_NO_WIN1256']

harfbuzz_inc = include_directories('src')

harfbuzz_sources = files('src/harfbuzz.cc')

harfbuzz_lib = library(
  'harfbuzz',
  harfbuzz_sources,
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
