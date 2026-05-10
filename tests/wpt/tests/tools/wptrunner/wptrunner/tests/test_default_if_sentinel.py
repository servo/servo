import dataclasses
import itertools

import pytest

from .._default_if_sentinel import DefaultIfSentinel


def test_descriptor_with_default() -> None:
    class A:
        x = DefaultIfSentinel(default="default")

    o = A()
    assert o.x == "default"

    o.x = "changed"
    assert o.x == "changed"

    o.x = None
    assert o.x == "default"


def test_descriptor_with_default_factory() -> None:
    class A:
        x = DefaultIfSentinel(default_factory=lambda: "default")

    o = A()
    assert o.x == "default"

    o.x = "changed"
    assert o.x == "changed"

    o.x = None
    assert o.x == "default"


@pytest.mark.parametrize(
    "sentinel_value,assigned_value",
    list(
        itertools.product(
            [
                None,
                object(),
                "SENTINEL",
                [],
            ],
            repeat=2,
        )
    ),
)
def test_descriptor_sentinel_handling(
    sentinel_value: object, assigned_value: object
) -> None:
    class TestClass:
        value = DefaultIfSentinel(default="default", sentinel=sentinel_value)

    obj = TestClass()
    obj.value = assigned_value

    if assigned_value is sentinel_value:
        assert obj.value == "default"
    else:
        assert obj.value is assigned_value


def test_descriptor_identity_not_equality() -> None:
    sentinel_list: list[int] = []

    class TestClass:
        value = DefaultIfSentinel(default_factory=lambda: [1], sentinel=sentinel_list)

    obj = TestClass()
    obj.value = sentinel_list
    assert obj.value is not sentinel_list
    assert obj.value == [1]

    different_list: list[int] = []
    obj.value = different_list
    assert obj.value is different_list


def test_descriptor_list_same_instance() -> None:
    class A:
        value: DefaultIfSentinel[list[int]] = DefaultIfSentinel(default_factory=list)

    obj = A()
    first = obj.value
    second = obj.value
    assert first is second


def test_descriptor_list_isolation_between_instances() -> None:
    class A:
        value: DefaultIfSentinel[list[int]] = DefaultIfSentinel(default_factory=list)

    obj1 = A()
    obj2 = A()
    assert obj1.value == []
    assert obj2.value == []
    assert obj1.value is not obj2.value


def test_descriptor_list_different_instance_from_class() -> None:
    class A:
        value: DefaultIfSentinel[list[int]] = DefaultIfSentinel(default_factory=list)

    first = A.value
    second = A.value
    assert first == []
    assert second == []
    assert first is not second

    A.value.append(1)
    obj = A()
    A.value.append(2)

    assert obj.value == []
    assert obj.value is not first
    assert obj.value is not second


def test_descriptor_dataclasses_field() -> None:
    @dataclasses.dataclass
    class TestClass:
        value: "DefaultIfSentinel[str]" = dataclasses.field(
            default=DefaultIfSentinel(default="default"),
            repr=False,
        )

    obj = TestClass()
    assert obj.value == "default"


def test_descriptor_dataclasses_frozen() -> None:
    @dataclasses.dataclass(frozen=True)
    class TestClass:
        value: "DefaultIfSentinel[str]" = DefaultIfSentinel(default="default")

    obj1 = TestClass()
    assert obj1.value == "default"

    obj2 = TestClass("changed")
    assert obj2.value == "changed"

    with pytest.raises(dataclasses.FrozenInstanceError):
        obj1.value = "changed"  # type: ignore[misc]
