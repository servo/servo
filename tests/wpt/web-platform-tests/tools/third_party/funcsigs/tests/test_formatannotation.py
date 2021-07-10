import funcsigs

import unittest2 as unittest

class TestFormatAnnotation(unittest.TestCase):
    def test_string (self):
        self.assertEqual(funcsigs.formatannotation("annotation"),
                         "'annotation'")

    def test_builtin_type (self):
        self.assertEqual(funcsigs.formatannotation(int),
                         "int")

    def test_user_type (self):
        class dummy (object): pass
        self.assertEqual(funcsigs.formatannotation(dummy),
                         "tests.test_formatannotation.dummy")
