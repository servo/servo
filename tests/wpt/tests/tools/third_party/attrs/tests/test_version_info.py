# SPDX-License-Identifier: MIT

from __future__ import absolute_import, division, print_function

import pytest

from attr import VersionInfo
from attr._compat import PY2


@pytest.fixture(name="vi")
def fixture_vi():
    return VersionInfo(19, 2, 0, "final")


class TestVersionInfo:
    def test_from_string_no_releaselevel(self, vi):
        """
        If there is no suffix, the releaselevel becomes "final" by default.
        """
        assert vi == VersionInfo._from_version_string("19.2.0")

    def test_suffix_is_preserved(self):
        """
        If there is a suffix, it's preserved.
        """
        assert (
            "dev0"
            == VersionInfo._from_version_string("19.2.0.dev0").releaselevel
        )

    @pytest.mark.skipif(
        PY2, reason="Python 2 is too YOLO to care about comparability."
    )
    @pytest.mark.parametrize("other", [(), (19, 2, 0, "final", "garbage")])
    def test_wrong_len(self, vi, other):
        """
        Comparing with a tuple that has the wrong length raises an error.
        """
        assert vi != other

        with pytest.raises(TypeError):
            vi < other

    @pytest.mark.parametrize("other", [[19, 2, 0, "final"]])
    def test_wrong_type(self, vi, other):
        """
        Only compare to other VersionInfos or tuples.
        """
        assert vi != other

    def test_order(self, vi):
        """
        Ordering works as expected.
        """
        assert vi < (20,)
        assert vi < (19, 2, 1)
        assert vi > (0,)
        assert vi <= (19, 2)
        assert vi >= (19, 2)
        assert vi > (19, 2, 0, "dev0")
        assert vi < (19, 2, 0, "post1")
