import sys
import unittest
sys.path.insert(0, '/Users/krunal/Git/servo/tests/wpt/harness/wptrunner/executors/')
import webdriver
import subprocess
class ServoProcess(object):
    def __init__(self):
            #self.path = "/Users/krunal/Git/servo"
        self.proc = None

    def __enter__(self):
        self.proc = subprocess.Popen(["./mach run --webdriver 7000 tests/html/about-mozilla.html"], shell=True)

    def __exit__(self, *args, **kwargs):
        self.proc.kill()

class TestStringMethods(unittest.TestCase):
  def test_get(self):
      proc = subprocess.Popen(["./mach run --webdriver 8000 tests/html/about-mozilla.html"], shell=True)
      session = webdriver.Session('127.0.0.1', 8000)
      session.start()
      session.url="http://google.com/about"
      a=self.assertEqual( session.url , "https://www.google.com/about/")
      if a:
          session.end()
          proc.kill()

  def test_get1(self):
      proc = subprocess.Popen(["./mach run --webdriver 6000 tests/html/about-mozilla.html"], shell=True)
      session = webdriver.Session('127.0.0.1', 6000)
      session.start()
      session.url="http://google.com"
      a=self.assertEqual( session.url , "https://www.google.com/")
      if a:
          session.end()
          proc.kill()

'''
  def test_get2(self):
    with ServoProcess():
      session = webdriver.Session('127.0.0.1', 7000)
      session.start()
      session.url="http://ncsu.edu"
      self.assertEqual( session.url , "https://www.ncsu.edu/")
      session.close()
'''

if __name__ == '__main__':
    unittest.main()
