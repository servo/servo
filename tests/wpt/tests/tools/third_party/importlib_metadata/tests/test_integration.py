# coding: utf-8

from __future__ import unicode_literals

import unittest
import packaging.requirements
import packaging.version

from . import fixtures
from importlib_metadata import (
    Distribution,
    _compat,
    version,
    )


class IntegrationTests(fixtures.DistInfoPkg, unittest.TestCase):

    def test_package_spec_installed(self):
        """
        Illustrate the recommended procedure to determine if
        a specified version of a package is installed.
        """
        def is_installed(package_spec):
            req = packaging.requirements.Requirement(package_spec)
            return version(req.name) in req.specifier

        assert is_installed('distinfo-pkg==1.0')
        assert is_installed('distinfo-pkg>=1.0,<2.0')
        assert not is_installed('distinfo-pkg<1.0')


class FinderTests(fixtures.Fixtures, unittest.TestCase):

    def test_finder_without_module(self):
        class ModuleFreeFinder(fixtures.NullFinder):
            """
            A finder without an __module__ attribute
            """
            def __getattribute__(self, name):
                if name == '__module__':
                    raise AttributeError(name)
                return super().__getattribute__(name)

        self.fixtures.enter_context(
            fixtures.install_finder(ModuleFreeFinder()))
        _compat.disable_stdlib_finder()


class LocalProjectTests(fixtures.LocalPackage, unittest.TestCase):
    def test_find_local(self):
        dist = Distribution._local()
        assert dist.metadata['Name'] == 'local-pkg'
        assert dist.version == '2.0.1'
