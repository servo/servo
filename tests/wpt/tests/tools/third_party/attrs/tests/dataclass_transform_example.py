# SPDX-License-Identifier: MIT

import attr


@attr.define()
class Define:
    a: str
    b: int


reveal_type(Define.__init__)  # noqa


@attr.define()
class DefineConverter:
    # mypy plugin adapts the "int" method signature, pyright does not
    with_converter: int = attr.field(converter=int)


reveal_type(DefineConverter.__init__)  # noqa


# mypy plugin supports attr.frozen, pyright does not
@attr.frozen()
class Frozen:
    a: str


d = Frozen("a")
d.a = "new"

reveal_type(d.a)  # noqa


# but pyright supports attr.define(frozen)
@attr.define(frozen=True)
class FrozenDefine:
    a: str


d2 = FrozenDefine("a")
d2.a = "new"

reveal_type(d2.a)  # noqa
