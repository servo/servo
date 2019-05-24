# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import urllib


def ext_pillar(_minion_id, _pillar, *_args):
    url = "https://raw.githubusercontent.com/servo/saltfs/master/admin/files/ssh/%s.pub"
    return {"ssh_keys": [urllib.urlopen(url % name).read() for name in [
        "jdm",
        "manishearth",
        "simonsapin",
    ]]}
