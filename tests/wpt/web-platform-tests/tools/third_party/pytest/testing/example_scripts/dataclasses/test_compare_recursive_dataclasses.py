from dataclasses import dataclass


@dataclass
class S:
    a: int
    b: str


@dataclass
class C:
    c: S
    d: S


@dataclass
class C2:
    e: C
    f: S


@dataclass
class C3:
    g: S
    h: C2
    i: str
    j: str


def test_recursive_dataclasses():
    left = C3(
        S(10, "ten"), C2(C(S(1, "one"), S(2, "two")), S(2, "three")), "equal", "left",
    )
    right = C3(
        S(20, "xxx"), C2(C(S(1, "one"), S(2, "yyy")), S(3, "three")), "equal", "right",
    )

    assert left == right
