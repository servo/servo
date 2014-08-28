# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.
from __future__ import unicode_literals

import sys
import unittest

from mozfile.mozfile import NamedTemporaryFile

from mach.config import (
    AbsolutePathType,
    BooleanType,
    ConfigProvider,
    ConfigSettings,
    IntegerType,
    PathType,
    PositiveIntegerType,
    RelativePathType,
    StringType,
)

from mozunit import main


if sys.version_info[0] == 3:
    str_type = str
else:
    str_type = basestring

CONFIG1 = r"""
[foo]

bar = bar_value
baz = /baz/foo.c
"""

CONFIG2 = r"""
[foo]

bar = value2
"""

class Provider1(ConfigProvider):
    @classmethod
    def _register_settings(cls):
        cls.register_setting('foo', 'bar', StringType)
        cls.register_setting('foo', 'baz', AbsolutePathType)

Provider1.register_settings()

class ProviderDuplicate(ConfigProvider):
    @classmethod
    def _register_settings(cls):
        cls.register_setting('dupesect', 'foo', StringType)
        cls.register_setting('dupesect', 'foo', StringType)

class TestConfigProvider(unittest.TestCase):
    def test_construct(self):
        s = Provider1.config_settings

        self.assertEqual(len(s), 1)
        self.assertIn('foo', s)

        self.assertEqual(len(s['foo']), 2)
        self.assertIn('bar', s['foo'])
        self.assertIn('baz', s['foo'])

    def test_duplicate_option(self):
        with self.assertRaises(Exception):
            ProviderDuplicate.register_settings()


class Provider2(ConfigProvider):
    @classmethod
    def _register_settings(cls):
        cls.register_setting('a', 'string', StringType)
        cls.register_setting('a', 'boolean', BooleanType)
        cls.register_setting('a', 'pos_int', PositiveIntegerType)
        cls.register_setting('a', 'int', IntegerType)
        cls.register_setting('a', 'abs_path', AbsolutePathType)
        cls.register_setting('a', 'rel_path', RelativePathType)
        cls.register_setting('a', 'path', PathType)

Provider2.register_settings()

class TestConfigSettings(unittest.TestCase):
    def test_empty(self):
        s = ConfigSettings()

        self.assertEqual(len(s), 0)
        self.assertNotIn('foo', s)

    def test_simple(self):
        s = ConfigSettings()
        s.register_provider(Provider1)

        self.assertEqual(len(s), 1)
        self.assertIn('foo', s)

        foo = s['foo']
        foo = s.foo

        self.assertEqual(len(foo), 2)

        self.assertIn('bar', foo)
        self.assertIn('baz', foo)

        foo['bar'] = 'value1'
        self.assertEqual(foo['bar'], 'value1')
        self.assertEqual(foo['bar'], 'value1')

    def test_assignment_validation(self):
        s = ConfigSettings()
        s.register_provider(Provider2)

        a = s.a

        # Assigning an undeclared setting raises.
        with self.assertRaises(KeyError):
            a.undefined = True

        with self.assertRaises(KeyError):
            a['undefined'] = True

        # Basic type validation.
        a.string = 'foo'
        a.string = 'foo'

        with self.assertRaises(TypeError):
            a.string = False

        a.boolean = True
        a.boolean = False

        with self.assertRaises(TypeError):
            a.boolean = 'foo'

        a.pos_int = 5
        a.pos_int = 0

        with self.assertRaises(ValueError):
            a.pos_int = -1

        with self.assertRaises(TypeError):
            a.pos_int = 'foo'

        a.int = 5
        a.int = 0
        a.int = -5

        with self.assertRaises(TypeError):
            a.int = 1.24

        with self.assertRaises(TypeError):
            a.int = 'foo'

        a.abs_path = '/home/gps'

        with self.assertRaises(ValueError):
            a.abs_path = 'home/gps'

        a.rel_path = 'home/gps'
        a.rel_path = './foo/bar'
        a.rel_path = 'foo.c'

        with self.assertRaises(ValueError):
            a.rel_path = '/foo/bar'

        a.path = '/home/gps'
        a.path = 'foo.c'
        a.path = 'foo/bar'
        a.path = './foo'

    def test_retrieval_type(self):
        s = ConfigSettings()
        s.register_provider(Provider2)

        a = s.a

        a.string = 'foo'
        a.boolean = True
        a.pos_int = 12
        a.int = -4
        a.abs_path = '/home/gps'
        a.rel_path = 'foo.c'
        a.path = './foo/bar'

        self.assertIsInstance(a.string, str_type)
        self.assertIsInstance(a.boolean, bool)
        self.assertIsInstance(a.pos_int, int)
        self.assertIsInstance(a.int, int)
        self.assertIsInstance(a.abs_path, str_type)
        self.assertIsInstance(a.rel_path, str_type)
        self.assertIsInstance(a.path, str_type)

    def test_file_reading_single(self):
        temp = NamedTemporaryFile(mode='wt')
        temp.write(CONFIG1)
        temp.flush()

        s = ConfigSettings()
        s.register_provider(Provider1)

        s.load_file(temp.name)

        self.assertEqual(s.foo.bar, 'bar_value')

    def test_file_reading_multiple(self):
        """Loading multiple files has proper overwrite behavior."""
        temp1 = NamedTemporaryFile(mode='wt')
        temp1.write(CONFIG1)
        temp1.flush()

        temp2 = NamedTemporaryFile(mode='wt')
        temp2.write(CONFIG2)
        temp2.flush()

        s = ConfigSettings()
        s.register_provider(Provider1)

        s.load_files([temp1.name, temp2.name])

        self.assertEqual(s.foo.bar, 'value2')

    def test_file_reading_missing(self):
        """Missing files should silently be ignored."""

        s = ConfigSettings()

        s.load_file('/tmp/foo.ini')

    def test_file_writing(self):
        s = ConfigSettings()
        s.register_provider(Provider2)

        s.a.string = 'foo'
        s.a.boolean = False

        temp = NamedTemporaryFile('wt')
        s.write(temp)
        temp.flush()

        s2 = ConfigSettings()
        s2.register_provider(Provider2)

        s2.load_file(temp.name)

        self.assertEqual(s.a.string, s2.a.string)
        self.assertEqual(s.a.boolean, s2.a.boolean)

    def test_write_pot(self):
        s = ConfigSettings()
        s.register_provider(Provider1)
        s.register_provider(Provider2)

        # Just a basic sanity test.
        temp = NamedTemporaryFile('wt')
        s.write_pot(temp)
        temp.flush()


if __name__ == '__main__':
    main()
