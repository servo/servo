# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.


def main(request, response):
    headers = []
    if b'Content-Type' in request.GET:
        headers += [(b'Content-Type', request.GET[b'Content-Type'])]
    with open('./resources/ahem/AHEM____.TTF', 'rb') as f:
        return 200, headers, f.read()
