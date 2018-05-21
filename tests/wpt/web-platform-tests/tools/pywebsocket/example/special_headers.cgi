#!/usr/bin/python

# Copyright 2014 Google Inc. All rights reserved.
#
# Use of this source code is governed by a BSD-style
# license that can be found in the COPYING file or at
# https://developers.google.com/open-source/licenses/bsd

"""CGI script sample for testing effect of HTTP headers on the origin page.

Note that CGI scripts don't work on the standalone pywebsocket running in TLS
mode.
"""


print """Content-type: text/html
Content-Security-Policy: connect-src self

<html>
<head>
<title></title>
</head>
<body>
<script>
var socket = new WebSocket("ws://example.com");
</script>
</body>
</html>"""
