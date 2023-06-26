# SPDX-License-Identifier: MIT

import re

from typing import Any, Dict, List, Tuple, Union

import attr
import attrs


# Typing via "type" Argument ---


@attr.s
class C:
    a = attr.ib(type=int)


c = C(1)
C(a=1)


@attr.s
class D:
    x = attr.ib(type=List[int])


@attr.s
class E:
    y = attr.ib(type="List[int]")


@attr.s
class F:
    z = attr.ib(type=Any)


# Typing via Annotations ---


@attr.s
class CC:
    a: int = attr.ib()


cc = CC(1)
CC(a=1)


@attr.s
class DD:
    x: List[int] = attr.ib()


@attr.s
class EE:
    y: "List[int]" = attr.ib()


@attr.s
class FF:
    z: Any = attr.ib()


@attrs.define
class FFF:
    z: int


FFF(1)


# Inheritance --


@attr.s
class GG(DD):
    y: str = attr.ib()


GG(x=[1], y="foo")


@attr.s
class HH(DD, EE):
    z: float = attr.ib()


HH(x=[1], y=[], z=1.1)


# same class
c == cc


# Exceptions
@attr.s(auto_exc=True)
class Error(Exception):
    x: int = attr.ib()


try:
    raise Error(1)
except Error as e:
    e.x
    e.args
    str(e)


@attrs.define
class Error2(Exception):
    x: int


try:
    raise Error2(1)
except Error as e:
    e.x
    e.args
    str(e)


# Converters
# XXX: Currently converters can only be functions so none of this works
# although the stubs should be correct.

# @attr.s
# class ConvCOptional:
#     x: Optional[int] = attr.ib(converter=attr.converters.optional(int))


# ConvCOptional(1)
# ConvCOptional(None)


# @attr.s
# class ConvCDefaultIfNone:
#     x: int = attr.ib(converter=attr.converters.default_if_none(42))


# ConvCDefaultIfNone(1)
# ConvCDefaultIfNone(None)


# @attr.s
# class ConvCToBool:
#     x: int = attr.ib(converter=attr.converters.to_bool)


# ConvCToBool(1)
# ConvCToBool(True)
# ConvCToBool("on")
# ConvCToBool("yes")
# ConvCToBool(0)
# ConvCToBool(False)
# ConvCToBool("n")


# Validators
@attr.s
class Validated:
    a = attr.ib(
        type=List[C],
        validator=attr.validators.deep_iterable(
            attr.validators.instance_of(C), attr.validators.instance_of(list)
        ),
    )
    a = attr.ib(
        type=Tuple[C],
        validator=attr.validators.deep_iterable(
            attr.validators.instance_of(C), attr.validators.instance_of(tuple)
        ),
    )
    b = attr.ib(
        type=List[C],
        validator=attr.validators.deep_iterable(
            attr.validators.instance_of(C)
        ),
    )
    c = attr.ib(
        type=Dict[C, D],
        validator=attr.validators.deep_mapping(
            attr.validators.instance_of(C),
            attr.validators.instance_of(D),
            attr.validators.instance_of(dict),
        ),
    )
    d = attr.ib(
        type=Dict[C, D],
        validator=attr.validators.deep_mapping(
            attr.validators.instance_of(C), attr.validators.instance_of(D)
        ),
    )
    e: str = attr.ib(validator=attr.validators.matches_re(re.compile(r"foo")))
    f: str = attr.ib(
        validator=attr.validators.matches_re(r"foo", flags=42, func=re.search)
    )

    # Test different forms of instance_of
    g: int = attr.ib(validator=attr.validators.instance_of(int))
    h: int = attr.ib(validator=attr.validators.instance_of((int,)))
    j: Union[int, str] = attr.ib(
        validator=attr.validators.instance_of((int, str))
    )
    k: Union[int, str, C] = attr.ib(
        validator=attrs.validators.instance_of((int, C, str))
    )


@attr.define
class Validated2:
    num: int = attr.field(validator=attr.validators.ge(0))


@attrs.define
class Validated3:
    num: int = attr.field(validator=attr.validators.ge(0))


with attr.validators.disabled():
    Validated2(num=-1)

with attrs.validators.disabled():
    Validated3(num=-1)

try:
    attr.validators.set_disabled(True)
    Validated2(num=-1)
finally:
    attr.validators.set_disabled(False)


# Custom repr()
@attr.s
class WithCustomRepr:
    a: int = attr.ib(repr=True)
    b: str = attr.ib(repr=False)
    c: str = attr.ib(repr=lambda value: "c is for cookie")
    d: bool = attr.ib(repr=str)


@attrs.define
class WithCustomRepr2:
    a: int = attrs.field(repr=True)
    b: str = attrs.field(repr=False)
    c: str = attrs.field(repr=lambda value: "c is for cookie")
    d: bool = attrs.field(repr=str)


# Check some of our own types
@attr.s(eq=True, order=False)
class OrderFlags:
    a: int = attr.ib(eq=False, order=False)
    b: int = attr.ib(eq=True, order=True)


# on_setattr hooks
@attr.s(on_setattr=attr.setters.validate)
class ValidatedSetter:
    a: int
    b: str = attr.ib(on_setattr=attr.setters.NO_OP)
    c: bool = attr.ib(on_setattr=attr.setters.frozen)
    d: int = attr.ib(on_setattr=[attr.setters.convert, attr.setters.validate])
    e: bool = attr.ib(
        on_setattr=attr.setters.pipe(
            attr.setters.convert, attr.setters.validate
        )
    )


@attrs.define(on_setattr=attr.setters.validate)
class ValidatedSetter2:
    a: int
    b: str = attrs.field(on_setattr=attrs.setters.NO_OP)
    c: bool = attrs.field(on_setattr=attrs.setters.frozen)
    d: int = attrs.field(
        on_setattr=[attrs.setters.convert, attrs.setters.validate]
    )
    e: bool = attrs.field(
        on_setattr=attrs.setters.pipe(
            attrs.setters.convert, attrs.setters.validate
        )
    )


# field_transformer
def ft_hook(cls: type, attribs: List[attr.Attribute]) -> List[attr.Attribute]:
    return attribs


# field_transformer
def ft_hook2(
    cls: type, attribs: List[attrs.Attribute]
) -> List[attrs.Attribute]:
    return attribs


@attr.s(field_transformer=ft_hook)
class TransformedAttrs:
    x: int


@attrs.define(field_transformer=ft_hook2)
class TransformedAttrs2:
    x: int


# Auto-detect
@attr.s(auto_detect=True)
class AutoDetect:
    x: int

    def __init__(self, x: int):
        self.x = x


# Provisional APIs
@attr.define(order=True)
class NGClass:
    x: int = attr.field(default=42)


ngc = NGClass(1)


@attr.mutable(slots=False)
class NGClass2:
    x: int


ngc2 = NGClass2(1)


@attr.frozen(str=True)
class NGFrozen:
    x: int


ngf = NGFrozen(1)

attr.fields(NGFrozen).x.evolve(eq=False)
a = attr.fields(NGFrozen).x
a.evolve(repr=False)


attrs.fields(NGFrozen).x.evolve(eq=False)
a = attrs.fields(NGFrozen).x
a.evolve(repr=False)


@attr.s(collect_by_mro=True)
class MRO:
    pass


@attr.s
class FactoryTest:
    a: List[int] = attr.ib(default=attr.Factory(list))
    b: List[Any] = attr.ib(default=attr.Factory(list, False))
    c: List[int] = attr.ib(default=attr.Factory((lambda s: s.a), True))


@attrs.define
class FactoryTest2:
    a: List[int] = attrs.field(default=attrs.Factory(list))
    b: List[Any] = attrs.field(default=attrs.Factory(list, False))
    c: List[int] = attrs.field(default=attrs.Factory((lambda s: s.a), True))


attrs.asdict(FactoryTest2())
attr.asdict(FactoryTest(), tuple_keys=True)


# Check match_args stub
@attr.s(match_args=False)
class MatchArgs:
    a: int = attr.ib()
    b: int = attr.ib()


attr.asdict(FactoryTest())
attr.asdict(FactoryTest(), retain_collection_types=False)


# Check match_args stub
@attrs.define(match_args=False)
class MatchArgs2:
    a: int
    b: int


# NG versions of asdict/astuple
attrs.asdict(MatchArgs2(1, 2))
attrs.astuple(MatchArgs2(1, 2))


def importing_from_attr() -> None:
    """
    Use a function to keep the ns clean.
    """
    from attr.converters import optional
    from attr.exceptions import FrozenError
    from attr.filters import include
    from attr.setters import frozen
    from attr.validators import and_

    assert optional and FrozenError and include and frozen and and_


def importing_from_attrs() -> None:
    """
    Use a function to keep the ns clean.
    """
    from attrs.converters import optional
    from attrs.exceptions import FrozenError
    from attrs.filters import include
    from attrs.setters import frozen
    from attrs.validators import and_

    assert optional and FrozenError and include and frozen and and_
