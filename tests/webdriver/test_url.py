# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import sys
import unittest
import os.path as path
import webdriver
import subprocess
from ServoProcess import ServoProcess
import server



def handler(self):
    return 200, [('Content-Type', 'text/html')], '<html><body>hi there</body></html>'

class TestStringMethods(unittest.TestCase):
    
    def test_get(self):
        #print("inside test_get")
        with ServoProcess():
            server.serve(handler)
            session = webdriver.Session('127.0.0.1', 7000)
            session.start()
            session.url = "http://localhost:8001"
            self.assertEqual(session.url, "http://localhost:8001/")
            #session.end()
            server.stop()
        #print("inside test_get END")


# if __name__ == '__main__':
#     unittest.main()    

suite = unittest.TestSuite()
for method in dir(TestStringMethods):
    if method.startswith("test"):
        suite.addTest(TestStringMethods(method))
unittest.TextTestRunner().run(suite)
