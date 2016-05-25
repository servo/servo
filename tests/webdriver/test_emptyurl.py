# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import webdriver
import server
from ServoProcess import ServoProcess


def handler(self):
    return 200, [('Content-Type', 'text/html')], '<html><body>hi there</body></html>'

with ServoProcess():
    server.serve(handler)
    session = webdriver.Session('127.0.0.1', 7000)
    session.start()
    try:
        session.url = ""
    except Exception, e:
        raise Exception("URL is empty")

    # try:
    #     assert type(session.url) is StringType, "name is not a string: %r" % name
    #     except AssertionError, e:
    #         raise Exception("URL is not a string!")
    # session.end()
    server.stop()
