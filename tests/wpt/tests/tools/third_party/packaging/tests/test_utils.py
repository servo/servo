# This file is dual licensed under the terms of the Apache License, Version
# 2.0, and the BSD License. See the LICENSE file in the root of this repository
# for complete details.

import pytest

from packaging.tags import Tag
from packaging.utils import (
    InvalidSdistFilename,
    InvalidWheelFilename,
    canonicalize_name,
    canonicalize_version,
    parse_sdist_filename,
    parse_wheel_filename,
)
from packaging.version import Version


@pytest.mark.parametrize(
    ("name", "expected"),
    [
        ("foo", "foo"),
        ("Foo", "foo"),
        ("fOo", "foo"),
        ("foo.bar", "foo-bar"),
        ("Foo.Bar", "foo-bar"),
        ("Foo.....Bar", "foo-bar"),
        ("foo_bar", "foo-bar"),
        ("foo___bar", "foo-bar"),
        ("foo-bar", "foo-bar"),
        ("foo----bar", "foo-bar"),
    ],
)
def test_canonicalize_name(name, expected):
    assert canonicalize_name(name) == expected


@pytest.mark.parametrize(
    ("version", "expected"),
    [
        (Version("1.4.0"), "1.4"),
        ("1.4.0", "1.4"),
        ("1.40.0", "1.40"),
        ("1.4.0.0.00.000.0000", "1.4"),
        ("1.0", "1"),
        ("1.0+abc", "1+abc"),
        ("1.0.dev0", "1.dev0"),
        ("1.0.post0", "1.post0"),
        ("1.0a0", "1a0"),
        ("1.0rc0", "1rc0"),
        ("100!0.0", "100!0"),
        ("1.0.1-test7", "1.0.1-test7"),  # LegacyVersion is unchanged
    ],
)
def test_canonicalize_version(version, expected):
    assert canonicalize_version(version) == expected


@pytest.mark.parametrize(
    ("filename", "name", "version", "build", "tags"),
    [
        (
            "foo-1.0-py3-none-any.whl",
            "foo",
            Version("1.0"),
            (),
            {Tag("py3", "none", "any")},
        ),
        (
            "some_PACKAGE-1.0-py3-none-any.whl",
            "some-package",
            Version("1.0"),
            (),
            {Tag("py3", "none", "any")},
        ),
        (
            "foo-1.0-1000-py3-none-any.whl",
            "foo",
            Version("1.0"),
            (1000, ""),
            {Tag("py3", "none", "any")},
        ),
        (
            "foo-1.0-1000abc-py3-none-any.whl",
            "foo",
            Version("1.0"),
            (1000, "abc"),
            {Tag("py3", "none", "any")},
        ),
    ],
)
def test_parse_wheel_filename(filename, name, version, build, tags):
    assert parse_wheel_filename(filename) == (name, version, build, tags)


@pytest.mark.parametrize(
    ("filename"),
    [
        ("foo-1.0.whl"),  # Missing tags
        ("foo-1.0-py3-none-any.wheel"),  # Incorrect file extension (`.wheel`)
        ("foo__bar-1.0-py3-none-any.whl"),  # Invalid name (`__`)
        ("foo#bar-1.0-py3-none-any.whl"),  # Invalid name (`#`)
        # Build number doesn't start with a digit (`abc`)
        ("foo-1.0-abc-py3-none-any.whl"),
        ("foo-1.0-200-py3-none-any-junk.whl"),  # Too many dashes (`-junk`)
    ],
)
def test_parse_wheel_invalid_filename(filename):
    with pytest.raises(InvalidWheelFilename):
        parse_wheel_filename(filename)


@pytest.mark.parametrize(
    ("filename", "name", "version"),
    [("foo-1.0.tar.gz", "foo", Version("1.0")), ("foo-1.0.zip", "foo", Version("1.0"))],
)
def test_parse_sdist_filename(filename, name, version):
    assert parse_sdist_filename(filename) == (name, version)


@pytest.mark.parametrize(("filename"), [("foo-1.0.xz"), ("foo1.0.tar.gz")])
def test_parse_sdist_invalid_filename(filename):
    with pytest.raises(InvalidSdistFilename):
        parse_sdist_filename(filename)
