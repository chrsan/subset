# subset

Subset is a lightweight text layout library.

Subset doesn't concern itself with full fledged text layout features such
as line breaking, hyphenation etc. That kind of stuff needs to be handled
elsewhere.

## Development

You'll need [Meson][1] and [Ninja][2] to be able to develop and build the C/C++ code.
Use your preferred package manager to install them.

### Setup and build the C/C++ archives

You don't need to do this step when building the bindings below.

```shell
meson setup build
cd build
meson compile -v
```

If you're on a macOS and get linker errors try using:

```shell
LDFLAGS="-ld_classic" meson setup build
```

### Build and install the Python bindings

```shell
cd bindings/python
pip install .
```

### Build the Rust bindings

You'll need [Meson][1] and [Ninja][2] to build the [Rust][3] bindings.

```shell
cd bindings/rust
cargo run --example bidi
```

[1]: https://mesonbuild.com
[2]: https://ninja-build.org
[3]: https://www.rust-lang.org
