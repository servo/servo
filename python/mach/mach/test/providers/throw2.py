# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# This file exists to trigger the differences in mach error reporting between
# exceptions that occur in mach command modules themselves and in the things
# they call.

def throw_deep(message):
    return throw_real(message)

def throw_real(message):
    raise Exception(message)
