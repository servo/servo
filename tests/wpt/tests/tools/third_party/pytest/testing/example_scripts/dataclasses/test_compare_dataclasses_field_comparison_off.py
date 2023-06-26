from dataclasses import dataclass
from dataclasses import field


def test_dataclasses_with_attribute_comparison_off() -> None:
    @dataclass
    class SimpleDataObject:
        field_a: int = field()
        field_b: str = field(compare=False)

    left = SimpleDataObject(1, "b")
    right = SimpleDataObject(1, "c")

    assert left == right
