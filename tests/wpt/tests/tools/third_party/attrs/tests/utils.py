# SPDX-License-Identifier: MIT

"""
Common helper functions for tests.
"""


from attr import Attribute
from attr._make import NOTHING, _default_init_alias_for, make_class


def simple_class(
    eq=False,
    order=False,
    repr=False,
    hash=False,
    str=False,
    slots=False,
    frozen=False,
    cache_hash=False,
):
    """
    Return a new simple class.
    """
    return make_class(
        "C",
        ["a", "b"],
        eq=eq or order,
        order=order,
        repr=repr,
        hash=hash,
        init=True,
        slots=slots,
        str=str,
        frozen=frozen,
        cache_hash=cache_hash,
    )


def simple_attr(
    name,
    default=NOTHING,
    validator=None,
    repr=True,
    eq=True,
    hash=None,
    init=True,
    converter=None,
    kw_only=False,
    inherited=False,
):
    """
    Return an attribute with a name and no other bells and whistles.
    """
    return Attribute(
        name=name,
        default=default,
        validator=validator,
        repr=repr,
        cmp=None,
        eq=eq,
        hash=hash,
        init=init,
        converter=converter,
        kw_only=kw_only,
        inherited=inherited,
        alias=_default_init_alias_for(name),
    )
