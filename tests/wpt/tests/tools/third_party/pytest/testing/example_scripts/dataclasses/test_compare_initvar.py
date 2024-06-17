# mypy: allow-untyped-defs
from dataclasses import dataclass
from dataclasses import InitVar


@dataclass
class Foo:
    init_only: InitVar[int]
    real_attr: int


def test_demonstrate():
    assert Foo(1, 2) == Foo(1, 3)
