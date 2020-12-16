# -*- coding: utf-8 -*-
from dataclasses import dataclass
from dataclasses import field


def test_dataclasses_verbose():
    @dataclass
    class SimpleDataObject(object):
        field_a: int = field()
        field_b: int = field()

    left = SimpleDataObject(1, "b")
    right = SimpleDataObject(1, "c")

    assert left == right
