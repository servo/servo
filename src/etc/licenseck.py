# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

license0="""\
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
"""

license1="""\
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
"""

license2="""\
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
"""

license3 = """\
// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
"""

license4 = """\
# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
"""

licenses = [license0, license1, license2, license3, license4]

exceptions = [
    "rust-http-client/http_parser.c", # BSD, Joyent
    "rust-http-client/http_parser.h", # BSD, Joyent
    "rust-opengles/gl2.h", # SGI Free Software B License Version 2.0, Khronos Group
    "rust-stb-image/stb_image.c", # Public domain
    "servo/dom/bindings/codegen/ply/ply/yacc.py", # BSD
    "servo/dom/bindings/codegen/ply/ply/__init__.py", # BSD
    "servo/dom/bindings/codegen/ply/ply/lex.py", # BSD
]

def check_license(name, contents):
    valid_license = False
    for a_valid_license in licenses:
        if contents.startswith(a_valid_license):
            valid_license = True
            break
    if valid_license:
        return True

    for exception in exceptions:
        if name.endswith(exception):
            return True

    firstlineish = contents[:100]
    if firstlineish.find("xfail-license") != -1:
        return True

    return False
