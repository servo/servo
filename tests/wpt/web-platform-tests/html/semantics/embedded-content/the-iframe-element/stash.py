# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


def main(request, response):
    if request.method == 'POST':
        request.server.stash.put(request.GET["id"], request.body)
        return ''
    return request.server.stash.take(request.GET["id"])
