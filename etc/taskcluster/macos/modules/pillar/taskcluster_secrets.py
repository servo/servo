# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys
sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "..", "packet.net"))
import tc


def ext_pillar(_minion_id, _pillar, *_args):
    tc.check()
    return tc.secret("project/servo/tc-client/worker/macos/1")
