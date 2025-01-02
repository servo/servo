# SPDX-License-Identifier: MIT

import attr
import attrs


@attr.define()
class Define:
    a: str
    b: int


reveal_type(Define.__init__)  # noqa: F821


@attr.define()
class DefineConverter:
    # mypy plugin adapts the "int" method signature, pyright does not
    with_converter: int = attr.field(converter=int)


reveal_type(DefineConverter.__init__)  # noqa: F821

DefineConverter(with_converter=b"42")


@attr.frozen()
class Frozen:
    a: str


d = Frozen("a")
d.a = "new"

reveal_type(d.a)  # noqa: F821


@attr.define(frozen=True)
class FrozenDefine:
    a: str


d2 = FrozenDefine("a")
d2.a = "new"

reveal_type(d2.a)  # noqa: F821


# Field-aliasing works
@attrs.define
class AliasedField:
    _a: int = attrs.field(alias="_a")


af = AliasedField(42)

reveal_type(af.__init__)  # noqa: F821


# unsafe_hash is accepted
@attrs.define(unsafe_hash=True)
class Hashable:
    pass
