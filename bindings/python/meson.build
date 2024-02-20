project('subset-python', 'c', 'cpp', 'cython')

subdir('deps/extern')
subdir('deps/lib')

py = import('python').find_installation(pure: false)

py.extension_module('lib', 'subset.pyx',
  cython_args: ['--module-name', 'lib'],
  dependencies: subset_lib_dep,
  install: true,
  subdir: 'subset',
)

python_sources = [
  '__init__.py',
  'layout.py',
  'lib.pyi',
  'types.py',
]

py.install_sources(python_sources,
  subdir: 'subset',
)
