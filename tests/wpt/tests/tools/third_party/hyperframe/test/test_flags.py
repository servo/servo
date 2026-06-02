# -*- coding: utf-8 -*-
from hyperframe.frame import (
    Flags, Flag,
)
import pytest


class TestFlags:
    def test_add(self):
        flags = Flags([Flag("VALID_FLAG", 0x00)])
        assert not flags

        flags.add("VALID_FLAG")
        flags.add("VALID_FLAG")
        assert "VALID_FLAG" in flags
        assert list(flags) == ["VALID_FLAG"]
        assert len(flags) == 1

    def test_remove(self):
        flags = Flags([Flag("VALID_FLAG", 0x00)])
        flags.add("VALID_FLAG")

        flags.discard("VALID_FLAG")
        assert "VALID_FLAG" not in flags
        assert list(flags) == []
        assert len(flags) == 0

        # discarding elements not in the set should not throw an exception
        flags.discard("END_STREAM")

    def test_validation(self):
        flags = Flags([Flag("VALID_FLAG", 0x00)])
        flags.add("VALID_FLAG")
        with pytest.raises(ValueError):
            flags.add("INVALID_FLAG")

    def test_repr(self):
        flags = Flags([Flag("VALID_FLAG", 0x00), Flag("OTHER_FLAG", 0x01)])
        assert repr(flags) == "[]"
        flags.add("VALID_FLAG")
        assert repr(flags) == "['VALID_FLAG']"
        flags.add("OTHER_FLAG")
        assert repr(flags) == "['OTHER_FLAG', 'VALID_FLAG']"
