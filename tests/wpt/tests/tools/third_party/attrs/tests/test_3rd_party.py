# SPDX-License-Identifier: MIT

"""
Tests for compatibility against other Python modules.
"""

import pytest

from hypothesis import given

from .strategies import simple_classes


cloudpickle = pytest.importorskip("cloudpickle")


class TestCloudpickleCompat(object):
    """
    Tests for compatibility with ``cloudpickle``.
    """

    @given(simple_classes())
    def test_repr(self, cls):
        """
        attrs instances can be pickled and un-pickled with cloudpickle.
        """
        inst = cls()
        # Exact values aren't a concern so long as neither direction
        # raises an exception.
        pkl = cloudpickle.dumps(inst)
        cloudpickle.loads(pkl)
