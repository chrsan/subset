project('subset-python', 'c', 'cpp', 'cython')

subset_proj = subproject(
  'subset',
  default_options: ['default_library=static'],
)

subset_dep = subset_proj.get_variable('subset_dep')

py = import('python').find_installation(pure: false)

py.extension_module(
  '_subset',
  'subset.pyx',
  cython_args: ['--module-name', '_subset'],
  dependencies: subset_dep,
  install: true,
  subdir: 'subset',
)

python_sources = files(
  '__init__.py',
  '_subset.pyi',
  '_types.py',
  'layout.py',
  'py.typed',
)

py.install_sources(
  python_sources,
  subdir: 'subset',
)
