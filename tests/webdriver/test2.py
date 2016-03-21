import sys
import unittest
sys.path.insert(0, '/Users/krunal/Git/servo/tests/wpt/harness/wptrunner/executors/')
import webdriver
import subprocess
class ServoProcess():
    def __init__(self):
            self.path = "/Users/krunal/Git/servo"
            self.proc = None
            print(self.path)

    def __enter__(self):
           self.proc = subprocess.Popen([self.path, "--webdriver", "7000"])

    def __exit__(self, *args, **kwargs):
          self.proc.kill()

class TestStringMethods(unittest.TestCase):

  def test_get(self):
    print("started")
    with ServoProcess():
      with webdriver.Session('127.0.0.1', 7000) as session:
        session.get("http://google.com")
        assert session.url == "http://google.com"

  def test_upper(self):
      self.assertEqual('foo'.upper(), 'FOO')

  def test_isupper(self):
      self.assertTrue('FOO'.isupper())
      self.assertFalse('Foo'.isupper())

  def test_split(self):
      s = 'hello world'
      self.assertEqual(s.split(), ['hello', 'world'])
      # check that s.split fails when the separator is not a string
      with self.assertRaises(TypeError):
          s.split(2)




if __name__ == '__main__':
    unittest.main()
