# WebKitGTK MiniBrowser

To be able to run tests with the [WebKitGTK](https://webkitgtk.org/)
MiniBrowser you need the following packages installed:

* Fedora: `webkitgtk6.0`
* Debian or Ubuntu: `webkitgtk-driver` or `webkit2gtk-driver`
* Arch: `webkitgtk-6.0`

The WebKitGTK MiniBrowser is not installed on the default binary path.
The `wpt` script will try to automatically locate it, but if you need
to run it manually you can find it on any of this paths:

* Fedora: `/usr/libexec/webkitgtk-${APIVERSION}/MiniBrowser`
* Arch: `/usr/lib/webkitgtk-${APIVERSION}/MiniBrowser`
* Debian or Ubuntu: `/usr/lib/${TRIPLET}/webkitgtk-${APIVERSION}/MiniBrowser`
  * Note: `${TRIPLET}` is the output of the command `gcc -dumpmachine`

# Nightly universal bundle

Alternatively you can pass to `wpt` the flags `--install-browser --channel=nightly`
and then `wpt` will automatically download the last bundle and unpack it on the
default `wpt` working directory (usually subdir `_venv3/browsers` in your `wpt` checkout)
Then it will use the unpacked `MiniBrowser` and `WebKitWebDriver` binaries to run the tests.

This universal bundles should work on any Linux distribution as they include inside
the tarball all the system libraries and resources needed to run WebKitGTK, from libc
up to the Mesa graphics drivers without requiring the usage of containers.

If you are using proprietary graphics drivers (NVIDIA, AMDGPU PRO, etc) and you experience
issues with this bundle then a possible workaround is to try to run the tests headless
inside a virtualized display like `Xvfb` (see command `xvfb-run -a` on Debian/Ubuntu).
You can do this also from inside a virtual machine or Docker container.

# Headless mode

WebKitGTK does not have a native headless mode, but you can workaround that
by running the tests inside a virtualized display. For example you can use
`weston` with the headless backend for a virtualized `Wayland` display,
or you can use `Xvfb` for a virtualized `X11` display.

Example:

    xvfb-run -a ./wpt run [more-options] webkitgtk_minibrowser [tests-to-run]


# Using a custom WebKitGTK build

If you want to test with a custom WebKitGTK build the easiest way is that you
install this build in a temporary directory (`/tmp/wkgtktest` in this example),
and then tell `wpt` to run it from there.

Steps:

1. Build WebKitGTK passing these arguments to `CMake`:

       -DENABLE_MINIBROWSER=ON -DCMAKE_INSTALL_PREFIX=/tmp/wkgtktest

2. Install it: `ninja install` (or `make install`)
3. Locate the `MiniBrowser` and `WebKitWebDriver` binaries under the install directory.
4. Run `wpt` passing these two paths like this:

       ./wpt run --webdriver-binary=/tmp/wkgtktest/bin/WebKitWebDriver \
                 --binary=/tmp/wkgtktest/libexec/MiniBrowser \
                 [more-options] webkitgtk_minibrowser [tests-to-run]

Note: It is important that you build WebKitGTK against the libraries of your system.
Do not build WebKitGTK inside Flatpak or other container unless you run `wpt` also
from inside this container.

# Running tests locally

Is a good idea that you increase the verbosity of `wpt` by passing to it the flag `--log-mach=-`
Also, please check the documentation about [Running Tests from the Local System](from-local-system).
