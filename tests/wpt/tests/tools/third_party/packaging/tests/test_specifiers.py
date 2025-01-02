# This file is dual licensed under the terms of the Apache License, Version
# 2.0, and the BSD License. See the LICENSE file in the root of this repository
# for complete details.

import itertools
import operator
import warnings

import pytest

from packaging.specifiers import (
    InvalidSpecifier,
    LegacySpecifier,
    Specifier,
    SpecifierSet,
)
from packaging.version import LegacyVersion, Version, parse

from .test_version import LEGACY_VERSIONS, VERSIONS

LEGACY_SPECIFIERS = [
    "==2.1.0.3",
    "!=2.2.0.5",
    "<=5",
    ">=7.9a1",
    "<1.0.dev1",
    ">2.0.post1",
]

SPECIFIERS = [
    "~=2.0",
    "==2.1.*",
    "==2.1.0.3",
    "!=2.2.*",
    "!=2.2.0.5",
    "<=5",
    ">=7.9a1",
    "<1.0.dev1",
    ">2.0.post1",
    "===lolwat",
]


class TestSpecifier:
    @pytest.mark.parametrize("specifier", SPECIFIERS)
    def test_specifiers_valid(self, specifier):
        Specifier(specifier)

    @pytest.mark.parametrize(
        "specifier",
        [
            # Operator-less specifier
            "2.0",
            # Invalid operator
            "=>2.0",
            # Version-less specifier
            "==",
            # Local segment on operators which don't support them
            "~=1.0+5",
            ">=1.0+deadbeef",
            "<=1.0+abc123",
            ">1.0+watwat",
            "<1.0+1.0",
            # Prefix matching on operators which don't support them
            "~=1.0.*",
            ">=1.0.*",
            "<=1.0.*",
            ">1.0.*",
            "<1.0.*",
            # Combination of local and prefix matching on operators which do
            # support one or the other
            "==1.0.*+5",
            "!=1.0.*+deadbeef",
            # Prefix matching cannot be used inside of a local version
            "==1.0+5.*",
            "!=1.0+deadbeef.*",
            # Prefix matching must appear at the end
            "==1.0.*.5",
            # Compatible operator requires 2 digits in the release operator
            "~=1",
            # Cannot use a prefix matching after a .devN version
            "==1.0.dev1.*",
            "!=1.0.dev1.*",
        ],
    )
    def test_specifiers_invalid(self, specifier):
        with pytest.raises(InvalidSpecifier):
            Specifier(specifier)

    @pytest.mark.parametrize(
        "version",
        [
            # Various development release incarnations
            "1.0dev",
            "1.0.dev",
            "1.0dev1",
            "1.0dev",
            "1.0-dev",
            "1.0-dev1",
            "1.0DEV",
            "1.0.DEV",
            "1.0DEV1",
            "1.0DEV",
            "1.0.DEV1",
            "1.0-DEV",
            "1.0-DEV1",
            # Various alpha incarnations
            "1.0a",
            "1.0.a",
            "1.0.a1",
            "1.0-a",
            "1.0-a1",
            "1.0alpha",
            "1.0.alpha",
            "1.0.alpha1",
            "1.0-alpha",
            "1.0-alpha1",
            "1.0A",
            "1.0.A",
            "1.0.A1",
            "1.0-A",
            "1.0-A1",
            "1.0ALPHA",
            "1.0.ALPHA",
            "1.0.ALPHA1",
            "1.0-ALPHA",
            "1.0-ALPHA1",
            # Various beta incarnations
            "1.0b",
            "1.0.b",
            "1.0.b1",
            "1.0-b",
            "1.0-b1",
            "1.0beta",
            "1.0.beta",
            "1.0.beta1",
            "1.0-beta",
            "1.0-beta1",
            "1.0B",
            "1.0.B",
            "1.0.B1",
            "1.0-B",
            "1.0-B1",
            "1.0BETA",
            "1.0.BETA",
            "1.0.BETA1",
            "1.0-BETA",
            "1.0-BETA1",
            # Various release candidate incarnations
            "1.0c",
            "1.0.c",
            "1.0.c1",
            "1.0-c",
            "1.0-c1",
            "1.0rc",
            "1.0.rc",
            "1.0.rc1",
            "1.0-rc",
            "1.0-rc1",
            "1.0C",
            "1.0.C",
            "1.0.C1",
            "1.0-C",
            "1.0-C1",
            "1.0RC",
            "1.0.RC",
            "1.0.RC1",
            "1.0-RC",
            "1.0-RC1",
            # Various post release incarnations
            "1.0post",
            "1.0.post",
            "1.0post1",
            "1.0post",
            "1.0-post",
            "1.0-post1",
            "1.0POST",
            "1.0.POST",
            "1.0POST1",
            "1.0POST",
            "1.0.POST1",
            "1.0-POST",
            "1.0-POST1",
            "1.0-5",
            # Local version case insensitivity
            "1.0+AbC"
            # Integer Normalization
            "1.01",
            "1.0a05",
            "1.0b07",
            "1.0c056",
            "1.0rc09",
            "1.0.post000",
            "1.1.dev09000",
            "00!1.2",
            "0100!0.0",
            # Various other normalizations
            "v1.0",
            "  \r \f \v v1.0\t\n",
        ],
    )
    def test_specifiers_normalized(self, version):
        if "+" not in version:
            ops = ["~=", "==", "!=", "<=", ">=", "<", ">"]
        else:
            ops = ["==", "!="]

        for op in ops:
            Specifier(op + version)

    @pytest.mark.parametrize(
        ("specifier", "expected"),
        [
            # Single item specifiers should just be reflexive
            ("!=2.0", "!=2.0"),
            ("<2.0", "<2.0"),
            ("<=2.0", "<=2.0"),
            ("==2.0", "==2.0"),
            (">2.0", ">2.0"),
            (">=2.0", ">=2.0"),
            ("~=2.0", "~=2.0"),
            # Spaces should be removed
            ("< 2", "<2"),
        ],
    )
    def test_specifiers_str_and_repr(self, specifier, expected):
        spec = Specifier(specifier)

        assert str(spec) == expected
        assert repr(spec) == f"<Specifier({expected!r})>"

    @pytest.mark.parametrize("specifier", SPECIFIERS)
    def test_specifiers_hash(self, specifier):
        assert hash(Specifier(specifier)) == hash(Specifier(specifier))

    @pytest.mark.parametrize(
        ("left", "right", "op"),
        itertools.chain(
            *
            # Verify that the equal (==) operator works correctly
            [[(x, x, operator.eq) for x in SPECIFIERS]]
            +
            # Verify that the not equal (!=) operator works correctly
            [
                [(x, y, operator.ne) for j, y in enumerate(SPECIFIERS) if i != j]
                for i, x in enumerate(SPECIFIERS)
            ]
        ),
    )
    def test_comparison_true(self, left, right, op):
        assert op(Specifier(left), Specifier(right))
        assert op(left, Specifier(right))
        assert op(Specifier(left), right)

    @pytest.mark.parametrize(("left", "right"), [("==2.8.0", "==2.8")])
    def test_comparison_canonicalizes(self, left, right):
        assert Specifier(left) == Specifier(right)
        assert left == Specifier(right)
        assert Specifier(left) == right

    @pytest.mark.parametrize(
        ("left", "right", "op"),
        itertools.chain(
            *
            # Verify that the equal (==) operator works correctly
            [[(x, x, operator.ne) for x in SPECIFIERS]]
            +
            # Verify that the not equal (!=) operator works correctly
            [
                [(x, y, operator.eq) for j, y in enumerate(SPECIFIERS) if i != j]
                for i, x in enumerate(SPECIFIERS)
            ]
        ),
    )
    def test_comparison_false(self, left, right, op):
        assert not op(Specifier(left), Specifier(right))
        assert not op(left, Specifier(right))
        assert not op(Specifier(left), right)

    def test_comparison_non_specifier(self):
        assert Specifier("==1.0") != 12
        assert not Specifier("==1.0") == 12
        assert Specifier("==1.0") != "12"
        assert not Specifier("==1.0") == "12"

    @pytest.mark.parametrize(
        ("version", "spec", "expected"),
        [
            (v, s, True)
            for v, s in [
                # Test the equality operation
                ("2.0", "==2"),
                ("2.0", "==2.0"),
                ("2.0", "==2.0.0"),
                ("2.0+deadbeef", "==2"),
                ("2.0+deadbeef", "==2.0"),
                ("2.0+deadbeef", "==2.0.0"),
                ("2.0+deadbeef", "==2+deadbeef"),
                ("2.0+deadbeef", "==2.0+deadbeef"),
                ("2.0+deadbeef", "==2.0.0+deadbeef"),
                ("2.0+deadbeef.0", "==2.0.0+deadbeef.00"),
                # Test the equality operation with a prefix
                ("2.dev1", "==2.*"),
                ("2a1", "==2.*"),
                ("2a1.post1", "==2.*"),
                ("2b1", "==2.*"),
                ("2b1.dev1", "==2.*"),
                ("2c1", "==2.*"),
                ("2c1.post1.dev1", "==2.*"),
                ("2rc1", "==2.*"),
                ("2", "==2.*"),
                ("2.0", "==2.*"),
                ("2.0.0", "==2.*"),
                ("2.0.post1", "==2.0.post1.*"),
                ("2.0.post1.dev1", "==2.0.post1.*"),
                ("2.1+local.version", "==2.1.*"),
                # Test the in-equality operation
                ("2.1", "!=2"),
                ("2.1", "!=2.0"),
                ("2.0.1", "!=2"),
                ("2.0.1", "!=2.0"),
                ("2.0.1", "!=2.0.0"),
                ("2.0", "!=2.0+deadbeef"),
                # Test the in-equality operation with a prefix
                ("2.0", "!=3.*"),
                ("2.1", "!=2.0.*"),
                # Test the greater than equal operation
                ("2.0", ">=2"),
                ("2.0", ">=2.0"),
                ("2.0", ">=2.0.0"),
                ("2.0.post1", ">=2"),
                ("2.0.post1.dev1", ">=2"),
                ("3", ">=2"),
                # Test the less than equal operation
                ("2.0", "<=2"),
                ("2.0", "<=2.0"),
                ("2.0", "<=2.0.0"),
                ("2.0.dev1", "<=2"),
                ("2.0a1", "<=2"),
                ("2.0a1.dev1", "<=2"),
                ("2.0b1", "<=2"),
                ("2.0b1.post1", "<=2"),
                ("2.0c1", "<=2"),
                ("2.0c1.post1.dev1", "<=2"),
                ("2.0rc1", "<=2"),
                ("1", "<=2"),
                # Test the greater than operation
                ("3", ">2"),
                ("2.1", ">2.0"),
                ("2.0.1", ">2"),
                ("2.1.post1", ">2"),
                ("2.1+local.version", ">2"),
                # Test the less than operation
                ("1", "<2"),
                ("2.0", "<2.1"),
                ("2.0.dev0", "<2.1"),
                # Test the compatibility operation
                ("1", "~=1.0"),
                ("1.0.1", "~=1.0"),
                ("1.1", "~=1.0"),
                ("1.9999999", "~=1.0"),
                ("1.1", "~=1.0a1"),
                # Test that epochs are handled sanely
                ("2!1.0", "~=2!1.0"),
                ("2!1.0", "==2!1.*"),
                ("2!1.0", "==2!1.0"),
                ("2!1.0", "!=1.0"),
                ("1.0", "!=2!1.0"),
                ("1.0", "<=2!0.1"),
                ("2!1.0", ">=2.0"),
                ("1.0", "<2!0.1"),
                ("2!1.0", ">2.0"),
                # Test some normalization rules
                ("2.0.5", ">2.0dev"),
            ]
        ]
        + [
            (v, s, False)
            for v, s in [
                # Test the equality operation
                ("2.1", "==2"),
                ("2.1", "==2.0"),
                ("2.1", "==2.0.0"),
                ("2.0", "==2.0+deadbeef"),
                # Test the equality operation with a prefix
                ("2.0", "==3.*"),
                ("2.1", "==2.0.*"),
                # Test the in-equality operation
                ("2.0", "!=2"),
                ("2.0", "!=2.0"),
                ("2.0", "!=2.0.0"),
                ("2.0+deadbeef", "!=2"),
                ("2.0+deadbeef", "!=2.0"),
                ("2.0+deadbeef", "!=2.0.0"),
                ("2.0+deadbeef", "!=2+deadbeef"),
                ("2.0+deadbeef", "!=2.0+deadbeef"),
                ("2.0+deadbeef", "!=2.0.0+deadbeef"),
                ("2.0+deadbeef.0", "!=2.0.0+deadbeef.00"),
                # Test the in-equality operation with a prefix
                ("2.dev1", "!=2.*"),
                ("2a1", "!=2.*"),
                ("2a1.post1", "!=2.*"),
                ("2b1", "!=2.*"),
                ("2b1.dev1", "!=2.*"),
                ("2c1", "!=2.*"),
                ("2c1.post1.dev1", "!=2.*"),
                ("2rc1", "!=2.*"),
                ("2", "!=2.*"),
                ("2.0", "!=2.*"),
                ("2.0.0", "!=2.*"),
                ("2.0.post1", "!=2.0.post1.*"),
                ("2.0.post1.dev1", "!=2.0.post1.*"),
                # Test the greater than equal operation
                ("2.0.dev1", ">=2"),
                ("2.0a1", ">=2"),
                ("2.0a1.dev1", ">=2"),
                ("2.0b1", ">=2"),
                ("2.0b1.post1", ">=2"),
                ("2.0c1", ">=2"),
                ("2.0c1.post1.dev1", ">=2"),
                ("2.0rc1", ">=2"),
                ("1", ">=2"),
                # Test the less than equal operation
                ("2.0.post1", "<=2"),
                ("2.0.post1.dev1", "<=2"),
                ("3", "<=2"),
                # Test the greater than operation
                ("1", ">2"),
                ("2.0.dev1", ">2"),
                ("2.0a1", ">2"),
                ("2.0a1.post1", ">2"),
                ("2.0b1", ">2"),
                ("2.0b1.dev1", ">2"),
                ("2.0c1", ">2"),
                ("2.0c1.post1.dev1", ">2"),
                ("2.0rc1", ">2"),
                ("2.0", ">2"),
                ("2.0.post1", ">2"),
                ("2.0.post1.dev1", ">2"),
                ("2.0+local.version", ">2"),
                # Test the less than operation
                ("2.0.dev1", "<2"),
                ("2.0a1", "<2"),
                ("2.0a1.post1", "<2"),
                ("2.0b1", "<2"),
                ("2.0b2.dev1", "<2"),
                ("2.0c1", "<2"),
                ("2.0c1.post1.dev1", "<2"),
                ("2.0rc1", "<2"),
                ("2.0", "<2"),
                ("2.post1", "<2"),
                ("2.post1.dev1", "<2"),
                ("3", "<2"),
                # Test the compatibility operation
                ("2.0", "~=1.0"),
                ("1.1.0", "~=1.0.0"),
                ("1.1.post1", "~=1.0.0"),
                # Test that epochs are handled sanely
                ("1.0", "~=2!1.0"),
                ("2!1.0", "~=1.0"),
                ("2!1.0", "==1.0"),
                ("1.0", "==2!1.0"),
                ("2!1.0", "==1.*"),
                ("1.0", "==2!1.*"),
                ("2!1.0", "!=2!1.0"),
            ]
        ],
    )
    def test_specifiers(self, version, spec, expected):
        spec = Specifier(spec, prereleases=True)

        if expected:
            # Test that the plain string form works
            assert version in spec
            assert spec.contains(version)

            # Test that the version instance form works
            assert Version(version) in spec
            assert spec.contains(Version(version))
        else:
            # Test that the plain string form works
            assert version not in spec
            assert not spec.contains(version)

            # Test that the version instance form works
            assert Version(version) not in spec
            assert not spec.contains(Version(version))

    @pytest.mark.parametrize(
        ("version", "spec", "expected"),
        [
            # Test identity comparison by itself
            ("lolwat", "===lolwat", True),
            ("Lolwat", "===lolwat", True),
            ("1.0", "===1.0", True),
            ("nope", "===lolwat", False),
            ("1.0.0", "===1.0", False),
            ("1.0.dev0", "===1.0.dev0", True),
        ],
    )
    def test_specifiers_identity(self, version, spec, expected):
        spec = Specifier(spec)

        if expected:
            # Identity comparisons only support the plain string form
            assert version in spec
        else:
            # Identity comparisons only support the plain string form
            assert version not in spec

    @pytest.mark.parametrize(
        ("specifier", "expected"),
        [
            ("==1.0", False),
            (">=1.0", False),
            ("<=1.0", False),
            ("~=1.0", False),
            ("<1.0", False),
            (">1.0", False),
            ("<1.0.dev1", False),
            (">1.0.dev1", False),
            ("==1.0.*", False),
            ("==1.0.dev1", True),
            (">=1.0.dev1", True),
            ("<=1.0.dev1", True),
            ("~=1.0.dev1", True),
        ],
    )
    def test_specifier_prereleases_detection(self, specifier, expected):
        assert Specifier(specifier).prereleases == expected

    @pytest.mark.parametrize(
        ("specifier", "version", "expected"),
        [
            (">=1.0", "2.0.dev1", False),
            (">=2.0.dev1", "2.0a1", True),
            ("==2.0.*", "2.0a1.dev1", False),
            ("==2.0a1.*", "2.0a1.dev1", True),
            ("<=2.0", "1.0.dev1", False),
            ("<=2.0.dev1", "1.0a1", True),
        ],
    )
    def test_specifiers_prereleases(self, specifier, version, expected):
        spec = Specifier(specifier)

        if expected:
            assert version in spec
            spec.prereleases = False
            assert version not in spec
        else:
            assert version not in spec
            spec.prereleases = True
            assert version in spec

    @pytest.mark.parametrize(
        ("specifier", "prereleases", "input", "expected"),
        [
            (">=1.0", None, ["2.0a1"], ["2.0a1"]),
            (">=1.0.dev1", None, ["1.0", "2.0a1"], ["1.0", "2.0a1"]),
            (">=1.0.dev1", False, ["1.0", "2.0a1"], ["1.0"]),
        ],
    )
    def test_specifier_filter(self, specifier, prereleases, input, expected):
        spec = Specifier(specifier)

        kwargs = {"prereleases": prereleases} if prereleases is not None else {}

        assert list(spec.filter(input, **kwargs)) == expected

    @pytest.mark.xfail
    def test_specifier_explicit_legacy(self):
        assert Specifier("==1.0").contains(LegacyVersion("1.0"))

    @pytest.mark.parametrize(
        ("spec", "op"),
        [
            ("~=2.0", "~="),
            ("==2.1.*", "=="),
            ("==2.1.0.3", "=="),
            ("!=2.2.*", "!="),
            ("!=2.2.0.5", "!="),
            ("<=5", "<="),
            (">=7.9a1", ">="),
            ("<1.0.dev1", "<"),
            (">2.0.post1", ">"),
            ("===lolwat", "==="),
        ],
    )
    def test_specifier_operator_property(self, spec, op):
        assert Specifier(spec).operator == op

    @pytest.mark.parametrize(
        ("spec", "version"),
        [
            ("~=2.0", "2.0"),
            ("==2.1.*", "2.1.*"),
            ("==2.1.0.3", "2.1.0.3"),
            ("!=2.2.*", "2.2.*"),
            ("!=2.2.0.5", "2.2.0.5"),
            ("<=5", "5"),
            (">=7.9a1", "7.9a1"),
            ("<1.0.dev1", "1.0.dev1"),
            (">2.0.post1", "2.0.post1"),
            ("===lolwat", "lolwat"),
        ],
    )
    def test_specifier_version_property(self, spec, version):
        assert Specifier(spec).version == version

    @pytest.mark.parametrize(
        ("spec", "expected_length"),
        [("", 0), ("==2.0", 1), (">=2.0", 1), (">=2.0,<3", 2), (">=2.0,<3,==2.4", 3)],
    )
    def test_length(self, spec, expected_length):
        spec = SpecifierSet(spec)
        assert len(spec) == expected_length

    @pytest.mark.parametrize(
        ("spec", "expected_items"),
        [
            ("", []),
            ("==2.0", ["==2.0"]),
            (">=2.0", [">=2.0"]),
            (">=2.0,<3", [">=2.0", "<3"]),
            (">=2.0,<3,==2.4", [">=2.0", "<3", "==2.4"]),
        ],
    )
    def test_iteration(self, spec, expected_items):
        spec = SpecifierSet(spec)
        items = {str(item) for item in spec}
        assert items == set(expected_items)


class TestLegacySpecifier:
    def test_legacy_specifier_is_deprecated(self):
        with warnings.catch_warnings(record=True) as w:
            LegacySpecifier(">=some-legacy-version")
            assert len(w) == 1
            assert issubclass(w[0].category, DeprecationWarning)

    @pytest.mark.parametrize(
        ("version", "spec", "expected"),
        [
            (v, s, True)
            for v, s in [
                # Test the equality operation
                ("2.0", "==2"),
                ("2.0", "==2.0"),
                ("2.0", "==2.0.0"),
                # Test the in-equality operation
                ("2.1", "!=2"),
                ("2.1", "!=2.0"),
                ("2.0.1", "!=2"),
                ("2.0.1", "!=2.0"),
                ("2.0.1", "!=2.0.0"),
                # Test the greater than equal operation
                ("2.0", ">=2"),
                ("2.0", ">=2.0"),
                ("2.0", ">=2.0.0"),
                ("2.0.post1", ">=2"),
                ("2.0.post1.dev1", ">=2"),
                ("3", ">=2"),
                # Test the less than equal operation
                ("2.0", "<=2"),
                ("2.0", "<=2.0"),
                ("2.0", "<=2.0.0"),
                ("2.0.dev1", "<=2"),
                ("2.0a1", "<=2"),
                ("2.0a1.dev1", "<=2"),
                ("2.0b1", "<=2"),
                ("2.0b1.post1", "<=2"),
                ("2.0c1", "<=2"),
                ("2.0c1.post1.dev1", "<=2"),
                ("2.0rc1", "<=2"),
                ("1", "<=2"),
                # Test the greater than operation
                ("3", ">2"),
                ("2.1", ">2.0"),
                # Test the less than operation
                ("1", "<2"),
                ("2.0", "<2.1"),
            ]
        ]
        + [
            (v, s, False)
            for v, s in [
                # Test the equality operation
                ("2.1", "==2"),
                ("2.1", "==2.0"),
                ("2.1", "==2.0.0"),
                # Test the in-equality operation
                ("2.0", "!=2"),
                ("2.0", "!=2.0"),
                ("2.0", "!=2.0.0"),
                # Test the greater than equal operation
                ("2.0.dev1", ">=2"),
                ("2.0a1", ">=2"),
                ("2.0a1.dev1", ">=2"),
                ("2.0b1", ">=2"),
                ("2.0b1.post1", ">=2"),
                ("2.0c1", ">=2"),
                ("2.0c1.post1.dev1", ">=2"),
                ("2.0rc1", ">=2"),
                ("1", ">=2"),
                # Test the less than equal operation
                ("2.0.post1", "<=2"),
                ("2.0.post1.dev1", "<=2"),
                ("3", "<=2"),
                # Test the greater than operation
                ("1", ">2"),
                ("2.0.dev1", ">2"),
                ("2.0a1", ">2"),
                ("2.0a1.post1", ">2"),
                ("2.0b1", ">2"),
                ("2.0b1.dev1", ">2"),
                ("2.0c1", ">2"),
                ("2.0c1.post1.dev1", ">2"),
                ("2.0rc1", ">2"),
                ("2.0", ">2"),
                # Test the less than operation
                ("3", "<2"),
            ]
        ],
    )
    def test_specifiers(self, version, spec, expected):
        spec = LegacySpecifier(spec, prereleases=True)

        if expected:
            # Test that the plain string form works
            assert version in spec
            assert spec.contains(version)

            # Test that the version instance form works
            assert LegacyVersion(version) in spec
            assert spec.contains(LegacyVersion(version))
        else:
            # Test that the plain string form works
            assert version not in spec
            assert not spec.contains(version)

            # Test that the version instance form works
            assert LegacyVersion(version) not in spec
            assert not spec.contains(LegacyVersion(version))

    def test_specifier_explicit_prereleases(self):
        spec = LegacySpecifier(">=1.0")
        assert not spec.prereleases
        spec.prereleases = True
        assert spec.prereleases

        spec = LegacySpecifier(">=1.0", prereleases=False)
        assert not spec.prereleases
        spec.prereleases = True
        assert spec.prereleases

        spec = LegacySpecifier(">=1.0", prereleases=True)
        assert spec.prereleases
        spec.prereleases = False
        assert not spec.prereleases

        spec = LegacySpecifier(">=1.0", prereleases=True)
        assert spec.prereleases
        spec.prereleases = None
        assert not spec.prereleases


class TestSpecifierSet:
    @pytest.mark.parametrize("version", VERSIONS + LEGACY_VERSIONS)
    def test_empty_specifier(self, version):
        spec = SpecifierSet(prereleases=True)

        assert version in spec
        assert spec.contains(version)
        assert parse(version) in spec
        assert spec.contains(parse(version))

    def test_specifier_prereleases_explicit(self):
        spec = SpecifierSet()
        assert not spec.prereleases
        assert "1.0.dev1" not in spec
        assert not spec.contains("1.0.dev1")
        spec.prereleases = True
        assert spec.prereleases
        assert "1.0.dev1" in spec
        assert spec.contains("1.0.dev1")

        spec = SpecifierSet(prereleases=True)
        assert spec.prereleases
        assert "1.0.dev1" in spec
        assert spec.contains("1.0.dev1")
        spec.prereleases = False
        assert not spec.prereleases
        assert "1.0.dev1" not in spec
        assert not spec.contains("1.0.dev1")

        spec = SpecifierSet(prereleases=True)
        assert spec.prereleases
        assert "1.0.dev1" in spec
        assert spec.contains("1.0.dev1")
        spec.prereleases = None
        assert not spec.prereleases
        assert "1.0.dev1" not in spec
        assert not spec.contains("1.0.dev1")

    def test_specifier_contains_prereleases(self):
        spec = SpecifierSet()
        assert spec.prereleases is None
        assert not spec.contains("1.0.dev1")
        assert spec.contains("1.0.dev1", prereleases=True)

        spec = SpecifierSet(prereleases=True)
        assert spec.prereleases
        assert spec.contains("1.0.dev1")
        assert not spec.contains("1.0.dev1", prereleases=False)

    @pytest.mark.parametrize(
        ("specifier", "specifier_prereleases", "prereleases", "input", "expected"),
        [
            # General test of the filter method
            ("", None, None, ["1.0", "2.0a1"], ["1.0"]),
            (">=1.0.dev1", None, None, ["1.0", "2.0a1"], ["1.0", "2.0a1"]),
            ("", None, None, ["1.0a1"], ["1.0a1"]),
            ("", None, None, ["1.0", Version("2.0")], ["1.0", Version("2.0")]),
            ("", None, None, ["2.0dog", "1.0"], ["1.0"]),
            # Test overriding with the prereleases parameter on filter
            ("", None, False, ["1.0a1"], []),
            (">=1.0.dev1", None, False, ["1.0", "2.0a1"], ["1.0"]),
            ("", None, True, ["1.0", "2.0a1"], ["1.0", "2.0a1"]),
            # Test overriding with the overall specifier
            ("", True, None, ["1.0", "2.0a1"], ["1.0", "2.0a1"]),
            ("", False, None, ["1.0", "2.0a1"], ["1.0"]),
            (">=1.0.dev1", True, None, ["1.0", "2.0a1"], ["1.0", "2.0a1"]),
            (">=1.0.dev1", False, None, ["1.0", "2.0a1"], ["1.0"]),
            ("", True, None, ["1.0a1"], ["1.0a1"]),
            ("", False, None, ["1.0a1"], []),
        ],
    )
    def test_specifier_filter(
        self, specifier_prereleases, specifier, prereleases, input, expected
    ):
        if specifier_prereleases is None:
            spec = SpecifierSet(specifier)
        else:
            spec = SpecifierSet(specifier, prereleases=specifier_prereleases)

        kwargs = {"prereleases": prereleases} if prereleases is not None else {}

        assert list(spec.filter(input, **kwargs)) == expected

    def test_legacy_specifiers_combined(self):
        spec = SpecifierSet("<3,>1-1-1")
        assert "2.0" in spec

    @pytest.mark.parametrize(
        ("specifier", "expected"),
        [
            # Single item specifiers should just be reflexive
            ("!=2.0", "!=2.0"),
            ("<2.0", "<2.0"),
            ("<=2.0", "<=2.0"),
            ("==2.0", "==2.0"),
            (">2.0", ">2.0"),
            (">=2.0", ">=2.0"),
            ("~=2.0", "~=2.0"),
            # Spaces should be removed
            ("< 2", "<2"),
            # Multiple item specifiers should work
            ("!=2.0,>1.0", "!=2.0,>1.0"),
            ("!=2.0 ,>1.0", "!=2.0,>1.0"),
        ],
    )
    def test_specifiers_str_and_repr(self, specifier, expected):
        spec = SpecifierSet(specifier)

        assert str(spec) == expected
        assert repr(spec) == f"<SpecifierSet({expected!r})>"

    @pytest.mark.parametrize("specifier", SPECIFIERS + LEGACY_SPECIFIERS)
    def test_specifiers_hash(self, specifier):
        assert hash(SpecifierSet(specifier)) == hash(SpecifierSet(specifier))

    @pytest.mark.parametrize(
        ("left", "right", "expected"), [(">2.0", "<5.0", ">2.0,<5.0")]
    )
    def test_specifiers_combine(self, left, right, expected):
        result = SpecifierSet(left) & SpecifierSet(right)
        assert result == SpecifierSet(expected)

        result = SpecifierSet(left) & right
        assert result == SpecifierSet(expected)

        result = SpecifierSet(left, prereleases=True) & SpecifierSet(right)
        assert result == SpecifierSet(expected)
        assert result.prereleases

        result = SpecifierSet(left, prereleases=False) & SpecifierSet(right)
        assert result == SpecifierSet(expected)
        assert not result.prereleases

        result = SpecifierSet(left) & SpecifierSet(right, prereleases=True)
        assert result == SpecifierSet(expected)
        assert result.prereleases

        result = SpecifierSet(left) & SpecifierSet(right, prereleases=False)
        assert result == SpecifierSet(expected)
        assert not result.prereleases

        result = SpecifierSet(left, prereleases=True) & SpecifierSet(
            right, prereleases=True
        )
        assert result == SpecifierSet(expected)
        assert result.prereleases

        result = SpecifierSet(left, prereleases=False) & SpecifierSet(
            right, prereleases=False
        )
        assert result == SpecifierSet(expected)
        assert not result.prereleases

        with pytest.raises(ValueError):
            result = SpecifierSet(left, prereleases=True) & SpecifierSet(
                right, prereleases=False
            )

        with pytest.raises(ValueError):
            result = SpecifierSet(left, prereleases=False) & SpecifierSet(
                right, prereleases=True
            )

    def test_specifiers_combine_not_implemented(self):
        with pytest.raises(TypeError):
            SpecifierSet() & 12

    @pytest.mark.parametrize(
        ("left", "right", "op"),
        itertools.chain(
            *
            # Verify that the equal (==) operator works correctly
            [[(x, x, operator.eq) for x in SPECIFIERS]]
            +
            # Verify that the not equal (!=) operator works correctly
            [
                [(x, y, operator.ne) for j, y in enumerate(SPECIFIERS) if i != j]
                for i, x in enumerate(SPECIFIERS)
            ]
        ),
    )
    def test_comparison_true(self, left, right, op):
        assert op(SpecifierSet(left), SpecifierSet(right))
        assert op(SpecifierSet(left), Specifier(right))
        assert op(Specifier(left), SpecifierSet(right))
        assert op(left, SpecifierSet(right))
        assert op(SpecifierSet(left), right)

    @pytest.mark.parametrize(
        ("left", "right", "op"),
        itertools.chain(
            *
            # Verify that the equal (==) operator works correctly
            [[(x, x, operator.ne) for x in SPECIFIERS]]
            +
            # Verify that the not equal (!=) operator works correctly
            [
                [(x, y, operator.eq) for j, y in enumerate(SPECIFIERS) if i != j]
                for i, x in enumerate(SPECIFIERS)
            ]
        ),
    )
    def test_comparison_false(self, left, right, op):
        assert not op(SpecifierSet(left), SpecifierSet(right))
        assert not op(SpecifierSet(left), Specifier(right))
        assert not op(Specifier(left), SpecifierSet(right))
        assert not op(left, SpecifierSet(right))
        assert not op(SpecifierSet(left), right)

    @pytest.mark.parametrize(("left", "right"), [("==2.8.0", "==2.8")])
    def test_comparison_canonicalizes(self, left, right):
        assert SpecifierSet(left) == SpecifierSet(right)
        assert left == SpecifierSet(right)
        assert SpecifierSet(left) == right

    def test_comparison_non_specifier(self):
        assert SpecifierSet("==1.0") != 12
        assert not SpecifierSet("==1.0") == 12

    @pytest.mark.parametrize(
        ("version", "specifier", "expected"),
        [
            ("1.0.0+local", "==1.0.0", True),
            ("1.0.0+local", "!=1.0.0", False),
            ("1.0.0+local", "<=1.0.0", True),
            ("1.0.0+local", ">=1.0.0", True),
            ("1.0.0+local", "<1.0.0", False),
            ("1.0.0+local", ">1.0.0", False),
        ],
    )
    def test_comparison_ignores_local(self, version, specifier, expected):
        assert (Version(version) in SpecifierSet(specifier)) == expected
