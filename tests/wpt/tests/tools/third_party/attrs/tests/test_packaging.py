# SPDX-License-Identifier: MIT

import sys

import pytest

import attr
import attrs


if sys.version_info < (3, 8):
    import importlib_metadata as metadata
else:
    from importlib import metadata


@pytest.fixture(name="mod", params=(attr, attrs))
def _mod(request):
    return request.param


class TestLegacyMetadataHack:
    def test_title(self, mod):
        """
        __title__ returns attrs.
        """
        with pytest.deprecated_call() as ws:
            assert "attrs" == mod.__title__

        assert (
            f"Accessing {mod.__name__}.__title__ is deprecated"
            in ws.list[0].message.args[0]
        )

    def test_copyright(self, mod):
        """
        __copyright__ returns the correct blurp.
        """
        with pytest.deprecated_call() as ws:
            assert "Copyright (c) 2015 Hynek Schlawack" == mod.__copyright__

        assert (
            f"Accessing {mod.__name__}.__copyright__ is deprecated"
            in ws.list[0].message.args[0]
        )

    def test_version(self, mod, recwarn):
        """
        __version__ returns the correct version and doesn't warn.
        """
        assert metadata.version("attrs") == mod.__version__

        assert [] == recwarn.list

    def test_description(self, mod):
        """
        __description__ returns the correct description.
        """
        with pytest.deprecated_call() as ws:
            assert "Classes Without Boilerplate" == mod.__description__

        assert (
            f"Accessing {mod.__name__}.__description__ is deprecated"
            in ws.list[0].message.args[0]
        )

    @pytest.mark.parametrize("name", ["__uri__", "__url__"])
    def test_uri(self, mod, name):
        """
        __uri__ & __url__ returns the correct project URL.
        """
        with pytest.deprecated_call() as ws:
            assert "https://www.attrs.org/" == getattr(mod, name)

        assert (
            f"Accessing {mod.__name__}.{name} is deprecated"
            in ws.list[0].message.args[0]
        )

    def test_author(self, mod):
        """
        __author__ returns Hynek.
        """
        with pytest.deprecated_call() as ws:
            assert "Hynek Schlawack" == mod.__author__

        assert (
            f"Accessing {mod.__name__}.__author__ is deprecated"
            in ws.list[0].message.args[0]
        )

    def test_email(self, mod):
        """
        __email__ returns Hynek's email address.
        """
        with pytest.deprecated_call() as ws:
            assert "hs@ox.cx" == mod.__email__

        assert (
            f"Accessing {mod.__name__}.__email__ is deprecated"
            in ws.list[0].message.args[0]
        )

    def test_license(self, mod):
        """
        __license__ returns MIT.
        """
        with pytest.deprecated_call() as ws:
            assert "MIT" == mod.__license__

        assert (
            f"Accessing {mod.__name__}.__license__ is deprecated"
            in ws.list[0].message.args[0]
        )

    def test_does_not_exist(self, mod):
        """
        Asking for unsupported dunders raises an AttributeError.
        """
        with pytest.raises(
            AttributeError,
            match=f"module {mod.__name__} has no attribute __yolo__",
        ):
            mod.__yolo__

    def test_version_info(self, recwarn, mod):
        """
        ___version_info__ is not deprected, therefore doesn't raise a warning
        and parses correctly.
        """
        assert isinstance(mod.__version_info__, attr.VersionInfo)
        assert [] == recwarn.list
