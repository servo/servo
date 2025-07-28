# Copyright 2025 The Chromium Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
import time

def main(request, response):
    delay = int(request.GET.first(b"delay")) / 1000
    time.sleep(delay)
    response.content = u"OK"
