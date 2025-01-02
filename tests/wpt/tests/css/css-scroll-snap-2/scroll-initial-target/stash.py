# Copyright 2023 The Chromium Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
"""
This file allows the different windows created by
css/css-scroll-snap-2/scroll-initial-target-with-text-fragment-navigation.html
to store and retrieve data.

scroll-initial-target-with-text-fragment-navigation.html (test file) opens a window to
scroll-initial-target-with-text-fragment-navigation-target.html which writes some data
which the test file will eventually read. This file handles the requests from
both windows.
"""

import time

def main(request, response):
    key = request.GET.first(b"key")

    if request.method == u"POST":
        # Received result data from target page
        request.server.stash.put(key, request.body, u'/css/css-scroll-snap-2/scroll-initial-target/')
        return u"ok"
    else:
        # Request for result data from test page
        value = request.server.stash.take(key, u'/css/css-scroll-snap-2/scroll-initial-target/')
        return value
