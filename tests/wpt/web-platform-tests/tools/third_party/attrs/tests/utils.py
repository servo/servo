"""
Common helper functions for tests.
"""

from __future__ import absolute_import, division, print_function

import keyword
import string

from collections import OrderedDict

from hypothesis import strategies as st

import attr

from attr import Attribute
from attr._make import NOTHING, make_class


def simple_class(cmp=False, repr=False, hash=False, str=False, slots=False,
                 frozen=False):
    """
    Return a new simple class.
    """
    return make_class(
        "C", ["a", "b"],
        cmp=cmp, repr=repr, hash=hash, init=True, slots=slots, str=str,
        frozen=frozen,
    )


def simple_attr(name, default=NOTHING, validator=None, repr=True,
                cmp=True, hash=None, init=True):
    """
    Return an attribute with a name and no other bells and whistles.
    """
    return Attribute(
        name=name, default=default, validator=validator, repr=repr,
        cmp=cmp, hash=hash, init=init
    )


class TestSimpleClass(object):
    """
    Tests for the testing helper function `make_class`.
    """
    def test_returns_class(self):
        """
        Returns a class object.
        """
        assert type is simple_class().__class__

    def returns_distinct_classes(self):
        """
        Each call returns a completely new class.
        """
        assert simple_class() is not simple_class()


def gen_attr_names():
    """
    Generate names for attributes, 'a'...'z', then 'aa'...'zz'.

    ~702 different attribute names should be enough in practice.

    Some short strings (such as 'as') are keywords, so we skip them.
    """
    lc = string.ascii_lowercase
    for c in lc:
        yield c
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
        yield val if not to_underscore else '_' + val
        to_underscore = not to_underscore


def _create_hyp_class(attrs):
    """
    A helper function for Hypothesis to generate attrs classes.
    """
    return make_class(
        "HypClass", dict(zip(gen_attr_names(), attrs))
    )


def _create_hyp_nested_strategy(simple_class_strategy):
    """
    Create a recursive attrs class.

    Given a strategy for building (simpler) classes, create and return
    a strategy for building classes that have as an attribute: either just
    the simpler class, a list of simpler classes, a tuple of simpler classes,
    an ordered dict or a dict mapping the string "cls" to a simpler class.
    """
    # Use a tuple strategy to combine simple attributes and an attr class.
    def just_class(tup):
        combined_attrs = list(tup[0])
        combined_attrs.append(attr.ib(default=attr.Factory(tup[1])))
        return _create_hyp_class(combined_attrs)

    def list_of_class(tup):
        default = attr.Factory(lambda: [tup[1]()])
        combined_attrs = list(tup[0])
        combined_attrs.append(attr.ib(default=default))
        return _create_hyp_class(combined_attrs)

    def tuple_of_class(tup):
        default = attr.Factory(lambda: (tup[1](),))
        combined_attrs = list(tup[0])
        combined_attrs.append(attr.ib(default=default))
        return _create_hyp_class(combined_attrs)

    def dict_of_class(tup):
        default = attr.Factory(lambda: {"cls": tup[1]()})
        combined_attrs = list(tup[0])
        combined_attrs.append(attr.ib(default=default))
        return _create_hyp_class(combined_attrs)

    def ordereddict_of_class(tup):
        default = attr.Factory(lambda: OrderedDict([("cls", tup[1]())]))
        combined_attrs = list(tup[0])
        combined_attrs.append(attr.ib(default=default))
        return _create_hyp_class(combined_attrs)

    # A strategy producing tuples of the form ([list of attributes], <given
    # class strategy>).
    attrs_and_classes = st.tuples(list_of_attrs, simple_class_strategy)

    return st.one_of(attrs_and_classes.map(just_class),
                     attrs_and_classes.map(list_of_class),
                     attrs_and_classes.map(tuple_of_class),
                     attrs_and_classes.map(dict_of_class),
                     attrs_and_classes.map(ordereddict_of_class))


bare_attrs = st.just(attr.ib(default=None))
int_attrs = st.integers().map(lambda i: attr.ib(default=i))
str_attrs = st.text().map(lambda s: attr.ib(default=s))
float_attrs = st.floats().map(lambda f: attr.ib(default=f))
dict_attrs = (st.dictionaries(keys=st.text(), values=st.integers())
              .map(lambda d: attr.ib(default=d)))

simple_attrs_without_metadata = (bare_attrs | int_attrs | str_attrs |
                                 float_attrs | dict_attrs)


@st.composite
def simple_attrs_with_metadata(draw):
    """
    Create a simple attribute with arbitrary metadata.
    """
    c_attr = draw(simple_attrs)
    keys = st.booleans() | st.binary() | st.integers() | st.text()
    vals = st.booleans() | st.binary() | st.integers() | st.text()
    metadata = draw(st.dictionaries(keys=keys, values=vals))

    return attr.ib(c_attr._default, c_attr._validator, c_attr.repr,
                   c_attr.cmp, c_attr.hash, c_attr.init, c_attr.convert,
                   metadata)


simple_attrs = simple_attrs_without_metadata | simple_attrs_with_metadata()

# Python functions support up to 255 arguments.
list_of_attrs = st.lists(simple_attrs, average_size=3, max_size=9)


@st.composite
def simple_classes(draw, slots=None, frozen=None, private_attrs=None):
    """
    A strategy that generates classes with default non-attr attributes.

    For example, this strategy might generate a class such as:

    @attr.s(slots=True, frozen=True)
    class HypClass:
        a = attr.ib(default=1)
        _b = attr.ib(default=None)
        c = attr.ib(default='text')
        _d = attr.ib(default=1.0)
        c = attr.ib(default={'t': 1})

    By default, all combinations of slots and frozen classes will be generated.
    If `slots=True` is passed in, only slots classes will be generated, and
    if `slots=False` is passed in, no slot classes will be generated. The same
    applies to `frozen`.

    By default, some attributes will be private (i.e. prefixed with an
    underscore). If `private_attrs=True` is passed in, all attributes will be
    private, and if `private_attrs=False`, no attributes will be private.
    """
    attrs = draw(list_of_attrs)
    frozen_flag = draw(st.booleans()) if frozen is None else frozen
    slots_flag = draw(st.booleans()) if slots is None else slots

    if private_attrs is None:
        attr_names = maybe_underscore_prefix(gen_attr_names())
    elif private_attrs is True:
        attr_names = ('_' + n for n in gen_attr_names())
    elif private_attrs is False:
        attr_names = gen_attr_names()

    cls_dict = dict(zip(attr_names, attrs))
    post_init_flag = draw(st.booleans())
    if post_init_flag:
        def post_init(self):
            pass
        cls_dict["__attrs_post_init__"] = post_init

    return make_class(
        "HypClass",
        cls_dict,
        slots=slots_flag,
        frozen=frozen_flag,
    )


# st.recursive works by taking a base strategy (in this case, simple_classes)
# and a special function.  This function receives a strategy, and returns
# another strategy (building on top of the base strategy).
nested_classes = st.recursive(
    simple_classes(),
    _create_hyp_nested_strategy,
    max_leaves=10
)
