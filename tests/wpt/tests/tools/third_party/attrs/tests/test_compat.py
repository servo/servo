# SPDX-License-Identifier: MIT

import types

import pytest

import attr


@pytest.fixture(name="mp")
def _mp():
    return types.MappingProxyType({"x": 42, "y": "foo"})


class TestMetadataProxy:
    """
    Ensure properties of metadata proxy independently of hypothesis strategies.
    """

    def test_repr(self, mp):
        """
        repr makes sense and is consistent across Python versions.
        """
        assert any(
            [
                "mappingproxy({'x': 42, 'y': 'foo'})" == repr(mp),
                "mappingproxy({'y': 'foo', 'x': 42})" == repr(mp),
            ]
        )

    def test_immutable(self, mp):
        """
        All mutating methods raise errors.
        """
        with pytest.raises(TypeError, match="not support item assignment"):
            mp["z"] = 23

        with pytest.raises(TypeError, match="not support item deletion"):
            del mp["x"]

        with pytest.raises(AttributeError, match="no attribute 'update'"):
            mp.update({})

        with pytest.raises(AttributeError, match="no attribute 'clear'"):
            mp.clear()

        with pytest.raises(AttributeError, match="no attribute 'pop'"):
            mp.pop("x")

        with pytest.raises(AttributeError, match="no attribute 'popitem'"):
            mp.popitem()

        with pytest.raises(AttributeError, match="no attribute 'setdefault'"):
            mp.setdefault("x")


def test_attrsinstance_subclass_protocol():
    """
    It's possible to subclass AttrsInstance and Protocol at once.
    """

    class Foo(attr.AttrsInstance, attr._compat.Protocol):
        def attribute(self) -> int:
            ...
