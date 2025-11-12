# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

OLD_MPL = """\
This Source Code Form is subject to the terms of the Mozilla Public \
License, v. 2.0. If a copy of the MPL was not distributed with this \
file, You can obtain one at http://mozilla.org/MPL/2.0/.\
"""

MPL = """\
This Source Code Form is subject to the terms of the Mozilla Public \
License, v. 2.0. If a copy of the MPL was not distributed with this \
file, You can obtain one at https://mozilla.org/MPL/2.0/.\
"""

APACHE = """\
Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or \
http://www.apache.org/licenses/LICENSE-2.0> or the MIT license \
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your \
option. This file may not be copied, modified, or distributed \
except according to those terms.\
"""

# List of accepted `COPYRIGHT` disclaimers.
COPYRIGHT = [
    "See the COPYRIGHT file at the top-level directory of this distribution",
]

# The valid licenses, in the form we'd expect to see them in a Cargo.toml file.
licenses_toml = [
    'license = "MPL-2.0"',
    'license = "MIT/Apache-2.0"',
    'license = "MIT OR Apache-2.0"',
]
