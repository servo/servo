import enum
from enum import Enum
from typing import TypeVar, Union


class Undefined(Enum):
    """
    Class representing special value that indicates that a property is not set.
    """

    UNDEFINED = enum.auto()


UNDEFINED = Undefined.UNDEFINED
"""A special value that indicates that a property is not set."""

T = TypeVar("T")

#: A type hint for a value that can be of a specific type or UNDEFINED.
#: For example, ``Maybe[str]`` is equivalent to ``Union[str, Undefined]``.
Maybe = Union[T, Undefined]

#: A type hint which can have protocol values `null`. Intended to be used
# instead of `Optional` to avoid confusion.
Nullable = Union[T, None]
