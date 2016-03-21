import unittest
from selenium import webdriver


class TestStringMethods(unittest.TestCase):

  def test_upper(self):
      self.assertEqual('foo'.upper(), 'FOOd')

  def test_isupper(self):
      self.assertTrue('FOO'.isupper())
      self.assertFalse('Foo'.isupper())

  def test_split(self):
      s = 'hello world'
      self.assertEqual(s.split(), ['hello', 'world'])
      # check that s.split fails when the separator is not a string
      with self.assertRaises(TypeError):
          s.split(2)

  def test_urlone(self):
      String URL = driver.getCurrentUrl();
      Assert.assertEquals(URL, "http://google.com" );

if __name__ == '__main__':
    unittest.main()
