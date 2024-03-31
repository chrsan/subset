# subset

Subset is a lightweight text layout library.

Subset doesn't concern itself with full fledged text layout features such
as line breaking, hyphenation etc. That kind of stuff needs to be handled
elsewhere.

## Development

You'll need [Meson][1] and [Ninja][2] to be able to develop and build the C/C++ code.
Use your preferred package manager to install them.

### Third party dependencies

You need to download some third party dependencies.

```shell
python3 ./tools/download_deps.py
```

### Setup and build the C/C++ archives

You don't need to do this step when building the bindings below.

```shell
meson setup build
cd build
meson compile -v
```

### Build and install the Python bindings

```shell
cd bindings/python
pip install .
```

[1]: https://mesonbuild.com
[2]: https://ninja-build.org
