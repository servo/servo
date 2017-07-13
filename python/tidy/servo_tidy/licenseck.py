# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

MPL = """\
This Source Code Form is subject to the terms of the Mozilla Public \
License, v. 2.0. If a copy of the MPL was not distributed with this \
file, You can obtain one at http://mozilla.org/MPL/2.0/.\
"""

APACHE = """\
Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or \
http://www.apache.org/licenses/LICENSE-2.0> or the MIT license \
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your \
option. This file may not be copied, modified, or distributed \
except according to those terms.\
"""

COPYRIGHT = [
    "See the COPYRIGHT file at the top-level directory of this distribution",
    "See http://rust-lang.org/COPYRIGHT",
]

# The valid licenses, in the form we'd expect to see them in a Cargo.toml file.
licenses_toml = [
    'license = "MPL-2.0"',
    'license = "MIT/Apache-2.0"',
]

# The valid dependency licenses, in the form we'd expect to see them in a Cargo.toml file.
licenses_dep_toml = [
    # Licenses that are compatible with Servo's licensing
    'license = "Apache-2 / MIT"',
    'license = "Apache-2.0 / MIT"',
    'license = "Apache-2.0"',
    'license = "Apache-2.0/MIT"',
    'license = "BSD-2-Clause"',
    'license = "BSD-3-Clause"',
    'license = "BSD-3-Clause/MIT"',
    'license = "CC0-1.0"',
    'license = "ISC"',
    'license = "MIT / Apache-2.0"',
    'license = "MIT OR Apache-2.0"',
    'license = "MIT"',
    'license = "MIT/Apache-2.0"',
    'license = "MPL-2.0"',
    'license = "Unlicense/MIT"',
    'license = "zlib-acknowledgement"',
    'license-file = "LICENSE-MIT"',
    'license=  "MIT / Apache-2.0"',
    # Whitelisted crates whose licensing has been checked manually
    'name = "device"',
    'name = "dylib"',
    'name = "ipc-channel"',
    'name = "mozjs_sys"',
    'name = "azure"',
    'name = "freetype"',
    'name = "js"',
    'name = "servo-freetype-sys"',
    'name = "simd"',
    'name = "webrender"',
    'name = "webrender_api"',
]
