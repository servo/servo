fn main() {
    ::std::os::set_exit_status(1);
    let _ = ::std::old_io::stderr().write(br"

    This is not the `Cargo.toml` file you're looking for.
    Invoke Cargo through mach instead, e.g. `./mach build`.

");
}
