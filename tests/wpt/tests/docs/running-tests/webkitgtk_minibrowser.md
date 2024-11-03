# WebKitGTK MiniBrowser


To be able to run tests with the WebKitGTK MiniBrowser you need the
following packages installed:

* Fedora: `webkit2gtk3-devel`
* Debian or Ubuntu: `webkit2gtk-driver`


The WebKitGTK MiniBrowser is not installed on the default binary path.
The `wpt` script will try to automatically locate it, but if you need
to run it manually you can find it on any of this paths:

* Fedora: `/usr/libexec/webkit2gtk-${VERSION}/MiniBrowser`
* Debian or Ubuntu: `/usr/lib/x86_64-linux-gnu/webkit2gtk-${VERSION}/MiniBrowser`
* Note:
     * `VERSION` is `4.0` or `4.1`.
     * If not Fedora and the machine architecture is not `x86_64`, then it will
      be located inside:
      `/usr/lib/${TRIPLET}/webkit2gtk-${VERSION}/MiniBrowser`
      where `TRIPLET=$(gcc -dumpmachine)`
