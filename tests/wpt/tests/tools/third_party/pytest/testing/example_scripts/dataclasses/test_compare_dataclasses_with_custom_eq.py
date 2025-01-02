from dataclasses import dataclass
from dataclasses import field


def test_dataclasses() -> None:
    @dataclass
    class SimpleDataObject:
        field_a: int = field()
        field_b: str = field()

        def __eq__(self, __o: object) -> bool:
            return super().__eq__(__o)

    left = SimpleDataObject(1, "b")
    right = SimpleDataObject(1, "c")

    assert left == right
