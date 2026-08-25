# SPDX-License-Identifier: MIT

"""
Testing strategies for Hypothesis-based tests.
"""
import functools
import keyword
import string

from collections import OrderedDict

from hypothesis import strategies as st

import attr

from attr._compat import PY_3_8_PLUS

from .utils import make_class


optional_bool = st.one_of(st.none(), st.booleans())


def gen_attr_names():
    """
    Generate names for attributes, 'a'...'z', then 'aa'...'zz'.

    ~702 different attribute names should be enough in practice.

    Some short strings (such as 'as') are keywords, so we skip them.
    """
    lc = string.ascii_lowercase
    yield from lc
    for outer in lc:
        for inner in lc:
            res = outer + inner
            if keyword.iskeyword(res):
                continue
            yield outer + inner


def maybe_underscore_prefix(source):
    """
    A generator to sometimes prepend an underscore.
    """
    to_underscore = False
    for val in source:
        yield val if not to_underscore else "_" + val
        to_underscore = not to_underscore


@st.composite
def _create_hyp_nested_strategy(draw, simple_class_strategy):
    """
    Create a recursive attrs class.

    Given a strategy for building (simpler) classes, create and return
    a strategy for building classes that have as an attribute: either just
    the simpler class, a list of simpler classes, a tuple of simpler classes,
    an ordered dict or a dict mapping the string "cls" to a simpler class.
    """
    cls = draw(simple_class_strategy)
    factories = [
        cls,
        lambda: [cls()],
        lambda: (cls(),),
        lambda: {"cls": cls()},
        lambda: OrderedDict([("cls", cls())]),
    ]
    factory = draw(st.sampled_from(factories))
    attrs = [*draw(list_of_attrs), attr.ib(default=attr.Factory(factory))]
    return make_class("HypClass", dict(zip(gen_attr_names(), attrs)))


bare_attrs = st.builds(attr.ib, default=st.none())
int_attrs = st.integers().map(lambda i: attr.ib(default=i))
str_attrs = st.text().map(lambda s: attr.ib(default=s))
float_attrs = st.floats().map(lambda f: attr.ib(default=f))
dict_attrs = st.dictionaries(keys=st.text(), values=st.integers()).map(
    lambda d: attr.ib(default=d)
)

simple_attrs_without_metadata = (
    bare_attrs | int_attrs | str_attrs | float_attrs | dict_attrs
)


@st.composite
def simple_attrs_with_metadata(draw):
    """
    Create a simple attribute with arbitrary metadata.
    """
    c_attr = draw(simple_attrs)
    keys = st.booleans() | st.binary() | st.integers() | st.text()
    vals = st.booleans() | st.binary() | st.integers() | st.text()
    metadata = draw(
        st.dictionaries(keys=keys, values=vals, min_size=1, max_size=3)
    )

    return attr.ib(
        default=c_attr._default,
        validator=c_attr._validator,
        repr=c_attr.repr,
        eq=c_attr.eq,
        order=c_attr.order,
        hash=c_attr.hash,
        init=c_attr.init,
        metadata=metadata,
        type=None,
        converter=c_attr.converter,
    )


simple_attrs = simple_attrs_without_metadata | simple_attrs_with_metadata()


# Python functions support up to 255 arguments.
list_of_attrs = st.lists(simple_attrs, max_size=3)


@st.composite
def simple_classes(
    draw,
    slots=None,
    frozen=None,
    weakref_slot=None,
    private_attrs=None,
    cached_property=None,
):
    """
    A strategy that generates classes with default non-attr attributes.

    For example, this strategy might generate a class such as:

    @attr.s(slots=True, frozen=True, weakref_slot=True)
    class HypClass:
        a = attr.ib(default=1)
        _b = attr.ib(default=None)
        c = attr.ib(default='text')
        _d = attr.ib(default=1.0)
        c = attr.ib(default={'t': 1})

    By default, all combinations of slots, frozen, and weakref_slot classes
    will be generated.  If `slots=True` is passed in, only slotted classes will
    be generated, and if `slots=False` is passed in, no slotted classes will be
    generated. The same applies to `frozen` and `weakref_slot`.

    By default, some attributes will be private (i.e. prefixed with an
    underscore). If `private_attrs=True` is passed in, all attributes will be
    private, and if `private_attrs=False`, no attributes will be private.
    """
    attrs = draw(list_of_attrs)
    frozen_flag = draw(st.booleans())
    slots_flag = draw(st.booleans())
    weakref_flag = draw(st.booleans())

    if private_attrs is None:
        attr_names = maybe_underscore_prefix(gen_attr_names())
    elif private_attrs is True:
        attr_names = ("_" + n for n in gen_attr_names())
    elif private_attrs is False:
        attr_names = gen_attr_names()

    cls_dict = dict(zip(attr_names, attrs))
    pre_init_flag = draw(st.booleans())
    post_init_flag = draw(st.booleans())
    init_flag = draw(st.booleans())
    cached_property_flag = draw(st.booleans())

    if pre_init_flag:

        def pre_init(self):
            pass

        cls_dict["__attrs_pre_init__"] = pre_init

    if post_init_flag:

        def post_init(self):
            pass

        cls_dict["__attrs_post_init__"] = post_init

    if not init_flag:

        def init(self, *args, **kwargs):
            self.__attrs_init__(*args, **kwargs)

        cls_dict["__init__"] = init

    bases = (object,)
    if cached_property or (
        PY_3_8_PLUS and cached_property is None and cached_property_flag
    ):

        class BaseWithCachedProperty:
            @functools.cached_property
            def _cached_property(self) -> int:
                return 1

        bases = (BaseWithCachedProperty,)

    return make_class(
        "HypClass",
        cls_dict,
        bases=bases,
        slots=slots_flag if slots is None else slots,
        frozen=frozen_flag if frozen is None else frozen,
        weakref_slot=weakref_flag if weakref_slot is None else weakref_slot,
        init=init_flag,
    )


# st.recursive works by taking a base strategy (in this case, simple_classes)
# and a special function.  This function receives a strategy, and returns
# another strategy (building on top of the base strategy).
nested_classes = st.recursive(
    simple_classes(), _create_hyp_nested_strategy, max_leaves=3
)
