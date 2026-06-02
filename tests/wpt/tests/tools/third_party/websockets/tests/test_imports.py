import types
import unittest
import warnings

from websockets.imports import *


foo = object()

bar = object()


class ImportsTests(unittest.TestCase):
    def setUp(self):
        self.mod = types.ModuleType("tests.test_imports.test_alias")
        self.mod.__package__ = self.mod.__name__

    def test_get_alias(self):
        lazy_import(
            vars(self.mod),
            aliases={"foo": "...test_imports"},
        )

        self.assertEqual(self.mod.foo, foo)

    def test_get_deprecated_alias(self):
        lazy_import(
            vars(self.mod),
            deprecated_aliases={"bar": "...test_imports"},
        )

        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            self.assertEqual(self.mod.bar, bar)

        self.assertEqual(len(recorded_warnings), 1)
        warning = recorded_warnings[0].message
        self.assertEqual(
            str(warning), "tests.test_imports.test_alias.bar is deprecated"
        )
        self.assertEqual(type(warning), DeprecationWarning)

    def test_dir(self):
        lazy_import(
            vars(self.mod),
            aliases={"foo": "...test_imports"},
            deprecated_aliases={"bar": "...test_imports"},
        )

        self.assertEqual(
            [item for item in dir(self.mod) if not item[:2] == item[-2:] == "__"],
            ["bar", "foo"],
        )

    def test_attribute_error(self):
        lazy_import(vars(self.mod))

        with self.assertRaises(AttributeError) as raised:
            self.mod.foo

        self.assertEqual(
            str(raised.exception),
            "module 'tests.test_imports.test_alias' has no attribute 'foo'",
        )
