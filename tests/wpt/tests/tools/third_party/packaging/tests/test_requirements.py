# This file is dual licensed under the terms of the Apache License, Version
# 2.0, and the BSD License. See the LICENSE file in the root of this repository
# for complete details.

import pytest

from packaging.markers import Marker
from packaging.requirements import URL, URL_AND_MARKER, InvalidRequirement, Requirement
from packaging.specifiers import SpecifierSet


class TestRequirements:
    def test_string_specifier_marker(self):
        requirement = 'name[bar]>=3; python_version == "2.7"'
        req = Requirement(requirement)
        assert str(req) == requirement

    def test_string_url(self):
        requirement = "name@ http://foo.com"
        req = Requirement(requirement)
        assert str(req) == requirement

    def test_string_url_with_marker(self):
        requirement = 'name@ http://foo.com ; extra == "feature"'
        req = Requirement(requirement)
        assert str(req) == requirement

    def test_repr(self):
        req = Requirement("name")
        assert repr(req) == "<Requirement('name')>"

    def _assert_requirement(
        self, req, name, url=None, extras=[], specifier="", marker=None
    ):
        assert req.name == name
        assert req.url == url
        assert sorted(req.extras) == sorted(extras)
        assert str(req.specifier) == specifier
        if marker:
            assert str(req.marker) == marker
        else:
            assert req.marker is None

    def test_simple_names(self):
        for name in ("A", "aa", "name"):
            req = Requirement(name)
            self._assert_requirement(req, name)

    def test_name_with_other_characters(self):
        name = "foo-bar.quux_baz"
        req = Requirement(name)
        self._assert_requirement(req, name)

    def test_invalid_name(self):
        with pytest.raises(InvalidRequirement):
            Requirement("foo!")

    def test_name_with_version(self):
        req = Requirement("name>=3")
        self._assert_requirement(req, "name", specifier=">=3")

    def test_with_legacy_version(self):
        req = Requirement("name==1.0.org1")
        self._assert_requirement(req, "name", specifier="==1.0.org1")

    def test_with_legacy_version_and_marker(self):
        req = Requirement("name>=1.x.y;python_version=='2.6'")
        self._assert_requirement(
            req, "name", specifier=">=1.x.y", marker='python_version == "2.6"'
        )

    def test_version_with_parens_and_whitespace(self):
        req = Requirement("name (==4)")
        self._assert_requirement(req, "name", specifier="==4")

    def test_name_with_multiple_versions(self):
        req = Requirement("name>=3,<2")
        self._assert_requirement(req, "name", specifier="<2,>=3")

    def test_name_with_multiple_versions_and_whitespace(self):
        req = Requirement("name >=2, <3")
        self._assert_requirement(req, "name", specifier="<3,>=2")

    def test_extras(self):
        req = Requirement("foobar [quux,bar]")
        self._assert_requirement(req, "foobar", extras=["bar", "quux"])

    def test_empty_extras(self):
        req = Requirement("foo[]")
        self._assert_requirement(req, "foo")

    def test_url(self):
        url_section = "@ http://example.com"
        parsed = URL.parseString(url_section)
        assert parsed.url == "http://example.com"

    def test_url_and_marker(self):
        instring = "@ http://example.com ; os_name=='a'"
        parsed = URL_AND_MARKER.parseString(instring)
        assert parsed.url == "http://example.com"
        assert str(parsed.marker) == 'os_name == "a"'

    def test_invalid_url(self):
        with pytest.raises(InvalidRequirement) as e:
            Requirement("name @ gopher:/foo/com")
        assert "Invalid URL: " in str(e.value)
        assert "gopher:/foo/com" in str(e.value)

    def test_file_url(self):
        req = Requirement("name @ file:///absolute/path")
        self._assert_requirement(req, "name", "file:///absolute/path")
        req = Requirement("name @ file://.")
        self._assert_requirement(req, "name", "file://.")

    def test_invalid_file_urls(self):
        with pytest.raises(InvalidRequirement):
            Requirement("name @ file:.")
        with pytest.raises(InvalidRequirement):
            Requirement("name @ file:/.")

    def test_extras_and_url_and_marker(self):
        req = Requirement("name [fred,bar] @ http://foo.com ; python_version=='2.7'")
        self._assert_requirement(
            req,
            "name",
            extras=["bar", "fred"],
            url="http://foo.com",
            marker='python_version == "2.7"',
        )

    def test_complex_url_and_marker(self):
        url = "https://example.com/name;v=1.1/?query=foo&bar=baz#blah"
        req = Requirement("foo @ %s ; python_version=='3.4'" % url)
        self._assert_requirement(req, "foo", url=url, marker='python_version == "3.4"')

    def test_multiple_markers(self):
        req = Requirement(
            "name[quux, strange];python_version<'2.7' and " "platform_version=='2'"
        )
        marker = 'python_version < "2.7" and platform_version == "2"'
        self._assert_requirement(req, "name", extras=["strange", "quux"], marker=marker)

    def test_multiple_comparison_markers(self):
        req = Requirement("name; os_name=='a' and os_name=='b' or os_name=='c'")
        marker = 'os_name == "a" and os_name == "b" or os_name == "c"'
        self._assert_requirement(req, "name", marker=marker)

    def test_invalid_marker(self):
        with pytest.raises(InvalidRequirement):
            Requirement("name; foobar=='x'")

    def test_types(self):
        req = Requirement("foobar[quux]<2,>=3; os_name=='a'")
        assert isinstance(req.name, str)
        assert isinstance(req.extras, set)
        assert req.url is None
        assert isinstance(req.specifier, SpecifierSet)
        assert isinstance(req.marker, Marker)

    def test_types_with_nothing(self):
        req = Requirement("foobar")
        assert isinstance(req.name, str)
        assert isinstance(req.extras, set)
        assert req.url is None
        assert isinstance(req.specifier, SpecifierSet)
        assert req.marker is None

    def test_types_with_url(self):
        req = Requirement("foobar @ http://foo.com")
        assert isinstance(req.name, str)
        assert isinstance(req.extras, set)
        assert isinstance(req.url, str)
        assert isinstance(req.specifier, SpecifierSet)
        assert req.marker is None

    def test_sys_platform_linux_equal(self):
        req = Requirement('something>=1.2.3; sys_platform == "foo"')

        assert req.name == "something"
        assert req.marker is not None
        assert req.marker.evaluate(dict(sys_platform="foo")) is True
        assert req.marker.evaluate(dict(sys_platform="bar")) is False

    def test_sys_platform_linux_in(self):
        req = Requirement("aviato>=1.2.3; 'f' in sys_platform")

        assert req.name == "aviato"
        assert req.marker is not None
        assert req.marker.evaluate(dict(sys_platform="foo")) is True
        assert req.marker.evaluate(dict(sys_platform="bar")) is False

    def test_parseexception_error_msg(self):
        with pytest.raises(InvalidRequirement) as e:
            Requirement("toto 42")
        assert "Expected stringEnd" in str(e.value) or (
            "Expected string_end" in str(e.value)  # pyparsing>=3.0.0
        )
