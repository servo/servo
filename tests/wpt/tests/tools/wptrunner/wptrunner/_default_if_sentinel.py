from __future__ import annotations

import sys
from collections.abc import Callable
from typing import (
    Generic,
    cast,
    overload,
)

if sys.version_info >= (3, 11):
    from typing import Self
else:
    from typing_extensions import Self

if sys.version_info >= (3, 13):
    from typing import TypeVar
else:
    from typing_extensions import TypeVar


_T = TypeVar("_T")
_S = TypeVar("_S", default=None)


class DefaultIfSentinel(Generic[_T, _S]):
    """Descriptor that returns a default value, reverting to it when the sentinel is assigned.

    ``DefaultIfSentinel[str]`` is a descriptor whose values are ``str`` and whose
    sentinel is ``None``::

        value: DefaultIfSentinel[str] = DefaultIfSentinel(default="hello")
        # obj.value is "hello" by default, and assigning None resets to "hello"

    ``DefaultIfSentinel[str, object]`` uses a dedicated sentinel object::

        UNSET = object()
        value: DefaultIfSentinel[str, object] = DefaultIfSentinel(
            default="hello", sentinel=UNSET
        )
        # obj.value is "hello" by default, and assigning UNSET resets to "hello"

    Type parameters:
        _T: Type of values that can be assigned
        _S: Type of the sentinel value (defaults to ``None``)

    """

    @overload
    def __init__(
        self: DefaultIfSentinel[_T, None], *, default: _T, sentinel: None = None
    ) -> None:
        ...

    @overload
    def __init__(
        self: DefaultIfSentinel[_T, None],
        *,
        default_factory: Callable[[], _T],
        sentinel: None = None,
    ) -> None:
        ...

    @overload
    def __init__(self, *, default: _T, sentinel: _S) -> None:
        ...

    @overload
    def __init__(self, *, default_factory: Callable[[], _T], sentinel: _S) -> None:
        ...

    def __init__(
        self,
        *,
        default: _T | None = None,
        default_factory: Callable[[], _T] | None = None,
        sentinel: _S | None = None,
    ) -> None:
        if default_factory is not None:
            self._default_factory = default_factory
        else:
            self._default_factory = cast("Callable[[], _T]", lambda: default)
        self._sentinel = sentinel

    def __set_name__(self, owner: type[object], name: str) -> None:
        self._name = "_" + name

    @overload
    def __get__(self, inst: None, owner: type[object]) -> _T:
        ...

    @overload
    def __get__(self, inst: object, owner: type[object] | None = None) -> _T:
        ...

    def __get__(self, inst: object | None, owner: type[object] | None = None) -> _T:
        if inst is None:
            return self._default_factory()

        value = cast("_T", getattr(inst, self._name, self._sentinel))
        if value is self._sentinel:
            value = self._default_factory()
            self.__set__(inst, value)
        return value

    def __set__(self, inst: object, value: _T | _S | Self) -> None:
        # dataclasses.field(default=Descriptor()) causes __set__ to be called with Self
        if value is self._sentinel or value is self:
            object.__setattr__(inst, self._name, self._default_factory())
        else:
            object.__setattr__(inst, self._name, value)
