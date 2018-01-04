from __future__ import absolute_import, division, print_function

import hashlib
import linecache
import sys

from operator import itemgetter

from . import _config
from ._compat import PY2, isclass, iteritems, metadata_proxy, set_closure_cell
from .exceptions import (
    DefaultAlreadySetError, FrozenInstanceError, NotAnAttrsClassError,
    UnannotatedAttributeError
)


# This is used at least twice, so cache it here.
_obj_setattr = object.__setattr__
_init_convert_pat = "__attr_convert_{}"
_init_factory_pat = "__attr_factory_{}"
_tuple_property_pat = "    {attr_name} = property(itemgetter({index}))"
_empty_metadata_singleton = metadata_proxy({})


class _Nothing(object):
    """
    Sentinel class to indicate the lack of a value when ``None`` is ambiguous.

    All instances of `_Nothing` are equal.
    """
    def __copy__(self):
        return self

    def __deepcopy__(self, _):
        return self

    def __eq__(self, other):
        return other.__class__ == _Nothing

    def __ne__(self, other):
        return not self == other

    def __repr__(self):
        return "NOTHING"

    def __hash__(self):
        return 0xdeadbeef


NOTHING = _Nothing()
"""
Sentinel to indicate the lack of a value when ``None`` is ambiguous.
"""


def attrib(default=NOTHING, validator=None,
           repr=True, cmp=True, hash=None, init=True,
           convert=None, metadata={}, type=None):
    """
    Create a new attribute on a class.

    ..  warning::

        Does *not* do anything unless the class is also decorated with
        :func:`attr.s`!

    :param default: A value that is used if an ``attrs``-generated ``__init__``
        is used and no value is passed while instantiating or the attribute is
        excluded using ``init=False``.

        If the value is an instance of :class:`Factory`, its callable will be
        used to construct a new value (useful for mutable data types like lists
        or dicts).

        If a default is not set (or set manually to ``attr.NOTHING``), a value
        *must* be supplied when instantiating; otherwise a :exc:`TypeError`
        will be raised.

        The default can also be set using decorator notation as shown below.

    :type default: Any value.

    :param validator: :func:`callable` that is called by ``attrs``-generated
        ``__init__`` methods after the instance has been initialized.  They
        receive the initialized instance, the :class:`Attribute`, and the
        passed value.

        The return value is *not* inspected so the validator has to throw an
        exception itself.

        If a ``list`` is passed, its items are treated as validators and must
        all pass.

        Validators can be globally disabled and re-enabled using
        :func:`get_run_validators`.

        The validator can also be set using decorator notation as shown below.

    :type validator: ``callable`` or a ``list`` of ``callable``\ s.

    :param bool repr: Include this attribute in the generated ``__repr__``
        method.
    :param bool cmp: Include this attribute in the generated comparison methods
        (``__eq__`` et al).
    :param hash: Include this attribute in the generated ``__hash__``
        method.  If ``None`` (default), mirror *cmp*'s value.  This is the
        correct behavior according the Python spec.  Setting this value to
        anything else than ``None`` is *discouraged*.
    :type hash: ``bool`` or ``None``
    :param bool init: Include this attribute in the generated ``__init__``
        method.  It is possible to set this to ``False`` and set a default
        value.  In that case this attributed is unconditionally initialized
        with the specified default value or factory.
    :param callable convert: :func:`callable` that is called by
        ``attrs``-generated ``__init__`` methods to convert attribute's value
        to the desired format.  It is given the passed-in value, and the
        returned value will be used as the new value of the attribute.  The
        value is converted before being passed to the validator, if any.
    :param metadata: An arbitrary mapping, to be used by third-party
        components.  See :ref:`extending_metadata`.
    :param type: The type of the attribute.  In Python 3.6 or greater, the
        preferred method to specify the type is using a variable annotation
        (see `PEP 526 <https://www.python.org/dev/peps/pep-0526/>`_).
        This argument is provided for backward compatibility.
        Regardless of the approach used, the type will be stored on
        ``Attribute.type``.

    ..  versionchanged:: 17.1.0 *validator* can be a ``list`` now.
    ..  versionchanged:: 17.1.0
        *hash* is ``None`` and therefore mirrors *cmp* by default.
    ..  versionadded:: 17.3.0 *type*
    """
    if hash is not None and hash is not True and hash is not False:
        raise TypeError(
            "Invalid value for hash.  Must be True, False, or None."
        )
    return _CountingAttr(
        default=default,
        validator=validator,
        repr=repr,
        cmp=cmp,
        hash=hash,
        init=init,
        convert=convert,
        metadata=metadata,
        type=type,
    )


def _make_attr_tuple_class(cls_name, attr_names):
    """
    Create a tuple subclass to hold `Attribute`s for an `attrs` class.

    The subclass is a bare tuple with properties for names.

    class MyClassAttributes(tuple):
        __slots__ = ()
        x = property(itemgetter(0))
    """
    attr_class_name = "{}Attributes".format(cls_name)
    attr_class_template = [
        "class {}(tuple):".format(attr_class_name),
        "    __slots__ = ()",
    ]
    if attr_names:
        for i, attr_name in enumerate(attr_names):
            attr_class_template.append(_tuple_property_pat.format(
                index=i,
                attr_name=attr_name,
            ))
    else:
        attr_class_template.append("    pass")
    globs = {"itemgetter": itemgetter}
    eval(compile("\n".join(attr_class_template), "", "exec"), globs)
    return globs[attr_class_name]


# Tuple class for extracted attributes from a class definition.
# `super_attrs` is a subset of `attrs`.
_Attributes = _make_attr_tuple_class("_Attributes", [
    "attrs",        # all attributes to build dunder methods for
    "super_attrs",  # attributes that have been inherited from super classes
])


def _is_class_var(annot):
    """
    Check whether *annot* is a typing.ClassVar.

    The implementation is gross but importing `typing` is slow and there are
    discussions to remove it from the stdlib alltogether.
    """
    return str(annot).startswith("typing.ClassVar")


def _get_annotations(cls):
    """
    Get annotations for *cls*.
    """
    anns = getattr(cls, "__annotations__", None)
    if anns is None:
        return {}

    # Verify that the annotations aren't merely inherited.
    for super_cls in cls.__mro__[1:]:
        if anns is getattr(super_cls, "__annotations__", None):
            return {}

    return anns


def _transform_attrs(cls, these, auto_attribs):
    """
    Transform all `_CountingAttr`s on a class into `Attribute`s.

    If *these* is passed, use that and don't look for them on the class.

    Return an `_Attributes`.
    """
    cd = cls.__dict__
    anns = _get_annotations(cls)

    if these is not None:
        ca_list = sorted((
            (name, ca)
            for name, ca
            in iteritems(these)
        ), key=lambda e: e[1].counter)
    elif auto_attribs is True:
        ca_names = {
            name
            for name, attr
            in cd.items()
            if isinstance(attr, _CountingAttr)
        }
        ca_list = []
        annot_names = set()
        for attr_name, type in anns.items():
            if _is_class_var(type):
                continue
            annot_names.add(attr_name)
            a = cd.get(attr_name, NOTHING)
            if not isinstance(a, _CountingAttr):
                if a is NOTHING:
                    a = attrib()
                else:
                    a = attrib(default=a)
            ca_list.append((attr_name, a))

        unannotated = ca_names - annot_names
        if len(unannotated) > 0:
            raise UnannotatedAttributeError(
                "The following `attr.ib`s lack a type annotation: " +
                ", ".join(sorted(
                    unannotated,
                    key=lambda n: cd.get(n).counter
                )) + "."
            )
    else:
        ca_list = sorted((
            (name, attr)
            for name, attr
            in cd.items()
            if isinstance(attr, _CountingAttr)
        ), key=lambda e: e[1].counter)

    non_super_attrs = [
        Attribute.from_counting_attr(
            name=attr_name,
            ca=ca,
            type=anns.get(attr_name),
        )
        for attr_name, ca
        in ca_list
    ]

    # Walk *down* the MRO for attributes.  While doing so, we collect the names
    # of attributes we've seen in `take_attr_names` and ignore their
    # redefinitions deeper in the hierarchy.
    super_attrs = []
    taken_attr_names = {a.name: a for a in non_super_attrs}
    for super_cls in cls.__mro__[1:-1]:
        sub_attrs = getattr(super_cls, "__attrs_attrs__", None)
        if sub_attrs is not None:
            # We iterate over sub_attrs backwards so we can reverse the whole
            # list in the end and get all attributes in the order they have
            # been defined.
            for a in reversed(sub_attrs):
                prev_a = taken_attr_names.get(a.name)
                if prev_a is None:
                    super_attrs.append(a)
                    taken_attr_names[a.name] = a
                elif prev_a == a:
                    # This happens thru multiple inheritance.  We don't want
                    # to favor attributes that are further down in the tree
                    # so we move them to the back.
                    super_attrs.remove(a)
                    super_attrs.append(a)

    # Now reverse the list, such that the attributes are sorted by *descending*
    # age.  IOW: the oldest attribute definition is at the head of the list.
    super_attrs.reverse()

    attr_names = [a.name for a in super_attrs + non_super_attrs]

    AttrsClass = _make_attr_tuple_class(cls.__name__, attr_names)

    attrs = AttrsClass(
        super_attrs + [
            Attribute.from_counting_attr(
                name=attr_name,
                ca=ca,
                type=anns.get(attr_name)
            )
            for attr_name, ca
            in ca_list
        ]
    )

    had_default = False
    for a in attrs:
        if had_default is True and a.default is NOTHING and a.init is True:
            raise ValueError(
                "No mandatory attributes allowed after an attribute with a "
                "default value or factory.  Attribute in question: {a!r}"
                .format(a=a)
            )
        elif had_default is False and \
                a.default is not NOTHING and \
                a.init is not False:
            had_default = True

    return _Attributes((attrs, super_attrs))


def _frozen_setattrs(self, name, value):
    """
    Attached to frozen classes as __setattr__.
    """
    raise FrozenInstanceError()


def _frozen_delattrs(self, name):
    """
    Attached to frozen classes as __delattr__.
    """
    raise FrozenInstanceError()


class _ClassBuilder(object):
    """
    Iteratively build *one* class.
    """
    __slots__ = (
        "_cls", "_cls_dict", "_attrs", "_super_names", "_attr_names", "_slots",
        "_frozen", "_has_post_init",
    )

    def __init__(self, cls, these, slots, frozen, auto_attribs):
        attrs, super_attrs = _transform_attrs(cls, these, auto_attribs)

        self._cls = cls
        self._cls_dict = dict(cls.__dict__) if slots else {}
        self._attrs = attrs
        self._super_names = set(a.name for a in super_attrs)
        self._attr_names = tuple(a.name for a in attrs)
        self._slots = slots
        self._frozen = frozen or _has_frozen_superclass(cls)
        self._has_post_init = bool(getattr(cls, "__attrs_post_init__", False))

        self._cls_dict["__attrs_attrs__"] = self._attrs

        if frozen:
            self._cls_dict["__setattr__"] = _frozen_setattrs
            self._cls_dict["__delattr__"] = _frozen_delattrs

    def __repr__(self):
        return "<_ClassBuilder(cls={cls})>".format(cls=self._cls.__name__)

    def build_class(self):
        """
        Finalize class based on the accumulated configuration.

        Builder cannot be used anymore after calling this method.
        """
        if self._slots is True:
            return self._create_slots_class()
        else:
            return self._patch_original_class()

    def _patch_original_class(self):
        """
        Apply accumulated methods and return the class.
        """
        cls = self._cls
        super_names = self._super_names

        # Clean class of attribute definitions (`attr.ib()`s).
        for name in self._attr_names:
            if name not in super_names and \
                    getattr(cls, name, None) is not None:
                delattr(cls, name)

        # Attach our dunder methods.
        for name, value in self._cls_dict.items():
            setattr(cls, name, value)

        return cls

    def _create_slots_class(self):
        """
        Build and return a new class with a `__slots__` attribute.
        """
        super_names = self._super_names
        cd = {
            k: v
            for k, v in iteritems(self._cls_dict)
            if k not in tuple(self._attr_names) + ("__dict__",)
        }

        # We only add the names of attributes that aren't inherited.
        # Settings __slots__ to inherited attributes wastes memory.
        cd["__slots__"] = tuple(
            name
            for name in self._attr_names
            if name not in super_names
        )

        qualname = getattr(self._cls, "__qualname__", None)
        if qualname is not None:
            cd["__qualname__"] = qualname

        attr_names = tuple(self._attr_names)

        def slots_getstate(self):
            """
            Automatically created by attrs.
            """
            return tuple(getattr(self, name) for name in attr_names)

        def slots_setstate(self, state):
            """
            Automatically created by attrs.
            """
            __bound_setattr = _obj_setattr.__get__(self, Attribute)
            for name, value in zip(attr_names, state):
                __bound_setattr(name, value)

        # slots and frozen require __getstate__/__setstate__ to work
        cd["__getstate__"] = slots_getstate
        cd["__setstate__"] = slots_setstate

        # Create new class based on old class and our methods.
        cls = type(self._cls)(
            self._cls.__name__,
            self._cls.__bases__,
            cd,
        )

        # The following is a fix for
        # https://github.com/python-attrs/attrs/issues/102.  On Python 3,
        # if a method mentions `__class__` or uses the no-arg super(), the
        # compiler will bake a reference to the class in the method itself
        # as `method.__closure__`.  Since we replace the class with a
        # clone, we rewrite these references so it keeps working.
        for item in cls.__dict__.values():
            if isinstance(item, (classmethod, staticmethod)):
                # Class- and staticmethods hide their functions inside.
                # These might need to be rewritten as well.
                closure_cells = getattr(item.__func__, "__closure__", None)
            else:
                closure_cells = getattr(item, "__closure__", None)

            if not closure_cells:  # Catch None or the empty list.
                continue
            for cell in closure_cells:
                if cell.cell_contents is self._cls:
                    set_closure_cell(cell, cls)

        return cls

    def add_repr(self, ns):
        self._cls_dict["__repr__"] = _make_repr(self._attrs, ns=ns)
        return self

    def add_str(self):
        repr_ = self._cls_dict.get("__repr__")
        if repr_ is None:
            raise ValueError(
                "__str__ can only be generated if a __repr__ exists."
            )

        self._cls_dict["__str__"] = repr_
        return self

    def make_unhashable(self):
        self._cls_dict["__hash__"] = None
        return self

    def add_hash(self):
        self._cls_dict["__hash__"] = _make_hash(self._attrs)
        return self

    def add_init(self):
        self._cls_dict["__init__"] = _make_init(
            self._attrs,
            self._has_post_init,
            self._frozen,
        )
        return self

    def add_cmp(self):
        cd = self._cls_dict

        cd["__eq__"], cd["__ne__"], cd["__lt__"], cd["__le__"], cd["__gt__"], \
            cd["__ge__"] = _make_cmp(self._attrs)

        return self


def attrs(maybe_cls=None, these=None, repr_ns=None,
          repr=True, cmp=True, hash=None, init=True,
          slots=False, frozen=False, str=False, auto_attribs=False):
    r"""
    A class decorator that adds `dunder
    <https://wiki.python.org/moin/DunderAlias>`_\ -methods according to the
    specified attributes using :func:`attr.ib` or the *these* argument.

    :param these: A dictionary of name to :func:`attr.ib` mappings.  This is
        useful to avoid the definition of your attributes within the class body
        because you can't (e.g. if you want to add ``__repr__`` methods to
        Django models) or don't want to.

        If *these* is not ``None``, ``attrs`` will *not* search the class body
        for attributes.

    :type these: :class:`dict` of :class:`str` to :func:`attr.ib`

    :param str repr_ns: When using nested classes, there's no way in Python 2
        to automatically detect that.  Therefore it's possible to set the
        namespace explicitly for a more meaningful ``repr`` output.
    :param bool repr: Create a ``__repr__`` method with a human readable
        representation of ``attrs`` attributes..
    :param bool str: Create a ``__str__`` method that is identical to
        ``__repr__``.  This is usually not necessary except for
        :class:`Exception`\ s.
    :param bool cmp: Create ``__eq__``, ``__ne__``, ``__lt__``, ``__le__``,
        ``__gt__``, and ``__ge__`` methods that compare the class as if it were
        a tuple of its ``attrs`` attributes.  But the attributes are *only*
        compared, if the type of both classes is *identical*!
    :param hash: If ``None`` (default), the ``__hash__`` method is generated
        according how *cmp* and *frozen* are set.

        1. If *both* are True, ``attrs`` will generate a ``__hash__`` for you.
        2. If *cmp* is True and *frozen* is False, ``__hash__`` will be set to
           None, marking it unhashable (which it is).
        3. If *cmp* is False, ``__hash__`` will be left untouched meaning the
           ``__hash__`` method of the superclass will be used (if superclass is
           ``object``, this means it will fall back to id-based hashing.).

        Although not recommended, you can decide for yourself and force
        ``attrs`` to create one (e.g. if the class is immutable even though you
        didn't freeze it programmatically) by passing ``True`` or not.  Both of
        these cases are rather special and should be used carefully.

        See the `Python documentation \
        <https://docs.python.org/3/reference/datamodel.html#object.__hash__>`_
        and the `GitHub issue that led to the default behavior \
        <https://github.com/python-attrs/attrs/issues/136>`_ for more details.
    :type hash: ``bool`` or ``None``
    :param bool init: Create a ``__init__`` method that initializes the
        ``attrs`` attributes.  Leading underscores are stripped for the
        argument name.  If a ``__attrs_post_init__`` method exists on the
        class, it will be called after the class is fully initialized.
    :param bool slots: Create a slots_-style class that's more
        memory-efficient.  See :ref:`slots` for further ramifications.
    :param bool frozen: Make instances immutable after initialization.  If
        someone attempts to modify a frozen instance,
        :exc:`attr.exceptions.FrozenInstanceError` is raised.

        Please note:

            1. This is achieved by installing a custom ``__setattr__`` method
               on your class so you can't implement an own one.

            2. True immutability is impossible in Python.

            3. This *does* have a minor a runtime performance :ref:`impact
               <how-frozen>` when initializing new instances.  In other words:
               ``__init__`` is slightly slower with ``frozen=True``.

            4. If a class is frozen, you cannot modify ``self`` in
               ``__attrs_post_init__`` or a self-written ``__init__``. You can
               circumvent that limitation by using
               ``object.__setattr__(self, "attribute_name", value)``.

        ..  _slots: https://docs.python.org/3/reference/datamodel.html#slots
    :param bool auto_attribs: If True, collect `PEP 526`_-annotated attributes
        (Python 3.6 and later only) from the class body.

        In this case, you **must** annotate every field.  If ``attrs``
        encounters a field that is set to an :func:`attr.ib` but lacks a type
        annotation, an :exc:`attr.exceptions.UnannotatedAttributeError` is
        raised.  Use ``field_name: typing.Any = attr.ib(...)`` if you don't
        want to set a type.

        If you assign a value to those attributes (e.g. ``x: int = 42``), that
        value becomes the default value like if it were passed using
        ``attr.ib(default=42)``.  Passing an instance of :class:`Factory` also
        works as expected.

        Attributes annotated as :data:`typing.ClassVar` are **ignored**.

        .. _`PEP 526`: https://www.python.org/dev/peps/pep-0526/

    ..  versionadded:: 16.0.0 *slots*
    ..  versionadded:: 16.1.0 *frozen*
    ..  versionadded:: 16.3.0 *str*, and support for ``__attrs_post_init__``.
    ..  versionchanged::
            17.1.0 *hash* supports ``None`` as value which is also the default
            now.
    .. versionadded:: 17.3.0 *auto_attribs*
    """
    def wrap(cls):
        if getattr(cls, "__class__", None) is None:
            raise TypeError("attrs only works with new-style classes.")

        builder = _ClassBuilder(cls, these, slots, frozen, auto_attribs)

        if repr is True:
            builder.add_repr(repr_ns)
        if str is True:
            builder.add_str()
        if cmp is True:
            builder.add_cmp()

        if hash is not True and hash is not False and hash is not None:
            # Can't use `hash in` because 1 == True for example.
            raise TypeError(
                "Invalid value for hash.  Must be True, False, or None."
            )
        elif hash is False or (hash is None and cmp is False):
            pass
        elif hash is True or (hash is None and cmp is True and frozen is True):
            builder.add_hash()
        else:
            builder.make_unhashable()

        if init is True:
            builder.add_init()

        return builder.build_class()

    # maybe_cls's type depends on the usage of the decorator.  It's a class
    # if it's used as `@attrs` but ``None`` if used as `@attrs()`.
    if maybe_cls is None:
        return wrap
    else:
        return wrap(maybe_cls)


_attrs = attrs
"""
Internal alias so we can use it in functions that take an argument called
*attrs*.
"""


if PY2:
    def _has_frozen_superclass(cls):
        """
        Check whether *cls* has a frozen ancestor by looking at its
        __setattr__.
        """
        return (
            getattr(
                cls.__setattr__, "__module__", None
            ) == _frozen_setattrs.__module__ and
            cls.__setattr__.__name__ == _frozen_setattrs.__name__
        )
else:
    def _has_frozen_superclass(cls):
        """
        Check whether *cls* has a frozen ancestor by looking at its
        __setattr__.
        """
        return cls.__setattr__ == _frozen_setattrs


def _attrs_to_tuple(obj, attrs):
    """
    Create a tuple of all values of *obj*'s *attrs*.
    """
    return tuple(getattr(obj, a.name) for a in attrs)


def _make_hash(attrs):
    attrs = tuple(
        a
        for a in attrs
        if a.hash is True or (a.hash is None and a.cmp is True)
    )

    # We cache the generated init methods for the same kinds of attributes.
    sha1 = hashlib.sha1()
    sha1.update(repr(attrs).encode("utf-8"))
    unique_filename = "<attrs generated hash %s>" % (sha1.hexdigest(),)
    type_hash = hash(unique_filename)
    lines = [
        "def __hash__(self):",
        "    return hash((",
        "        %d," % (type_hash,),
    ]
    for a in attrs:
        lines.append("        self.%s," % (a.name))

    lines.append("    ))")

    script = "\n".join(lines)
    globs = {}
    locs = {}
    bytecode = compile(script, unique_filename, "exec")
    eval(bytecode, globs, locs)

    # In order of debuggers like PDB being able to step through the code,
    # we add a fake linecache entry.
    linecache.cache[unique_filename] = (
        len(script),
        None,
        script.splitlines(True),
        unique_filename,
    )

    return locs["__hash__"]


def _add_hash(cls, attrs):
    """
    Add a hash method to *cls*.
    """
    cls.__hash__ = _make_hash(attrs)
    return cls


def _make_cmp(attrs):
    attrs = [a for a in attrs if a.cmp]

    def attrs_to_tuple(obj):
        """
        Save us some typing.
        """
        return _attrs_to_tuple(obj, attrs)

    def eq(self, other):
        """
        Automatically created by attrs.
        """
        if other.__class__ is self.__class__:
            return attrs_to_tuple(self) == attrs_to_tuple(other)
        else:
            return NotImplemented

    def ne(self, other):
        """
        Automatically created by attrs.
        """
        result = eq(self, other)
        if result is NotImplemented:
            return NotImplemented
        else:
            return not result

    def lt(self, other):
        """
        Automatically created by attrs.
        """
        if isinstance(other, self.__class__):
            return attrs_to_tuple(self) < attrs_to_tuple(other)
        else:
            return NotImplemented

    def le(self, other):
        """
        Automatically created by attrs.
        """
        if isinstance(other, self.__class__):
            return attrs_to_tuple(self) <= attrs_to_tuple(other)
        else:
            return NotImplemented

    def gt(self, other):
        """
        Automatically created by attrs.
        """
        if isinstance(other, self.__class__):
            return attrs_to_tuple(self) > attrs_to_tuple(other)
        else:
            return NotImplemented

    def ge(self, other):
        """
        Automatically created by attrs.
        """
        if isinstance(other, self.__class__):
            return attrs_to_tuple(self) >= attrs_to_tuple(other)
        else:
            return NotImplemented

    return eq, ne, lt, le, gt, ge


def _add_cmp(cls, attrs=None):
    """
    Add comparison methods to *cls*.
    """
    if attrs is None:
        attrs = cls.__attrs_attrs__

    cls.__eq__, cls.__ne__, cls.__lt__, cls.__le__, cls.__gt__, cls.__ge__ = \
        _make_cmp(attrs)

    return cls


def _make_repr(attrs, ns):
    """
    Make a repr method for *attr_names* adding *ns* to the full name.
    """
    attr_names = tuple(
        a.name
        for a in attrs
        if a.repr
    )

    def repr_(self):
        """
        Automatically created by attrs.
        """
        real_cls = self.__class__
        if ns is None:
            qualname = getattr(real_cls, "__qualname__", None)
            if qualname is not None:
                class_name = qualname.rsplit(">.", 1)[-1]
            else:
                class_name = real_cls.__name__
        else:
            class_name = ns + "." + real_cls.__name__

        return "{0}({1})".format(
            class_name,
            ", ".join(
                name + "=" + repr(getattr(self, name))
                for name in attr_names
            )
        )
    return repr_


def _add_repr(cls, ns=None, attrs=None):
    """
    Add a repr method to *cls*.
    """
    if attrs is None:
        attrs = cls.__attrs_attrs__

    repr_ = _make_repr(attrs, ns)
    cls.__repr__ = repr_
    return cls


def _make_init(attrs, post_init, frozen):
    attrs = [
        a
        for a in attrs
        if a.init or a.default is not NOTHING
    ]

    # We cache the generated init methods for the same kinds of attributes.
    sha1 = hashlib.sha1()
    sha1.update(repr(attrs).encode("utf-8"))
    unique_filename = "<attrs generated init {0}>".format(
        sha1.hexdigest()
    )

    script, globs = _attrs_to_init_script(
        attrs,
        frozen,
        post_init,
    )
    locs = {}
    bytecode = compile(script, unique_filename, "exec")
    attr_dict = dict((a.name, a) for a in attrs)
    globs.update({
        "NOTHING": NOTHING,
        "attr_dict": attr_dict,
    })
    if frozen is True:
        # Save the lookup overhead in __init__ if we need to circumvent
        # immutability.
        globs["_cached_setattr"] = _obj_setattr
    eval(bytecode, globs, locs)

    # In order of debuggers like PDB being able to step through the code,
    # we add a fake linecache entry.
    linecache.cache[unique_filename] = (
        len(script),
        None,
        script.splitlines(True),
        unique_filename,
    )

    return locs["__init__"]


def _add_init(cls, frozen):
    """
    Add a __init__ method to *cls*.  If *frozen* is True, make it immutable.
    """
    cls.__init__ = _make_init(
        cls.__attrs_attrs__,
        getattr(cls, "__attrs_post_init__", False),
        frozen,
    )
    return cls


def fields(cls):
    """
    Returns the tuple of ``attrs`` attributes for a class.

    The tuple also allows accessing the fields by their names (see below for
    examples).

    :param type cls: Class to introspect.

    :raise TypeError: If *cls* is not a class.
    :raise attr.exceptions.NotAnAttrsClassError: If *cls* is not an ``attrs``
        class.

    :rtype: tuple (with name accessors) of :class:`attr.Attribute`

    ..  versionchanged:: 16.2.0 Returned tuple allows accessing the fields
        by name.
    """
    if not isclass(cls):
        raise TypeError("Passed object must be a class.")
    attrs = getattr(cls, "__attrs_attrs__", None)
    if attrs is None:
        raise NotAnAttrsClassError(
            "{cls!r} is not an attrs-decorated class.".format(cls=cls)
        )
    return attrs


def validate(inst):
    """
    Validate all attributes on *inst* that have a validator.

    Leaves all exceptions through.

    :param inst: Instance of a class with ``attrs`` attributes.
    """
    if _config._run_validators is False:
        return

    for a in fields(inst.__class__):
        v = a.validator
        if v is not None:
            v(inst, a, getattr(inst, a.name))


def _attrs_to_init_script(attrs, frozen, post_init):
    """
    Return a script of an initializer for *attrs* and a dict of globals.

    The globals are expected by the generated script.

     If *frozen* is True, we cannot set the attributes directly so we use
    a cached ``object.__setattr__``.
    """
    lines = []
    if frozen is True:
        lines.append(
            # Circumvent the __setattr__ descriptor to save one lookup per
            # assignment.
            "_setattr = _cached_setattr.__get__(self, self.__class__)"
        )

        def fmt_setter(attr_name, value_var):
            return "_setattr('%(attr_name)s', %(value_var)s)" % {
                "attr_name": attr_name,
                "value_var": value_var,
            }

        def fmt_setter_with_converter(attr_name, value_var):
            conv_name = _init_convert_pat.format(attr_name)
            return "_setattr('%(attr_name)s', %(conv)s(%(value_var)s))" % {
                "attr_name": attr_name,
                "value_var": value_var,
                "conv": conv_name,
            }
    else:
        def fmt_setter(attr_name, value):
            return "self.%(attr_name)s = %(value)s" % {
                "attr_name": attr_name,
                "value": value,
            }

        def fmt_setter_with_converter(attr_name, value_var):
            conv_name = _init_convert_pat.format(attr_name)
            return "self.%(attr_name)s = %(conv)s(%(value_var)s)" % {
                "attr_name": attr_name,
                "value_var": value_var,
                "conv": conv_name,
            }

    args = []
    attrs_to_validate = []

    # This is a dictionary of names to validator and converter callables.
    # Injecting this into __init__ globals lets us avoid lookups.
    names_for_globals = {}

    for a in attrs:
        if a.validator:
            attrs_to_validate.append(a)
        attr_name = a.name
        arg_name = a.name.lstrip("_")
        has_factory = isinstance(a.default, Factory)
        if has_factory and a.default.takes_self:
            maybe_self = "self"
        else:
            maybe_self = ""
        if a.init is False:
            if has_factory:
                init_factory_name = _init_factory_pat.format(a.name)
                if a.convert is not None:
                    lines.append(fmt_setter_with_converter(
                        attr_name,
                        init_factory_name + "({0})".format(maybe_self)))
                    conv_name = _init_convert_pat.format(a.name)
                    names_for_globals[conv_name] = a.convert
                else:
                    lines.append(fmt_setter(
                        attr_name,
                        init_factory_name + "({0})".format(maybe_self)
                    ))
                names_for_globals[init_factory_name] = a.default.factory
            else:
                if a.convert is not None:
                    lines.append(fmt_setter_with_converter(
                        attr_name,
                        "attr_dict['{attr_name}'].default"
                        .format(attr_name=attr_name)
                    ))
                    conv_name = _init_convert_pat.format(a.name)
                    names_for_globals[conv_name] = a.convert
                else:
                    lines.append(fmt_setter(
                        attr_name,
                        "attr_dict['{attr_name}'].default"
                        .format(attr_name=attr_name)
                    ))
        elif a.default is not NOTHING and not has_factory:
            args.append(
                "{arg_name}=attr_dict['{attr_name}'].default".format(
                    arg_name=arg_name,
                    attr_name=attr_name,
                )
            )
            if a.convert is not None:
                lines.append(fmt_setter_with_converter(attr_name, arg_name))
                names_for_globals[_init_convert_pat.format(a.name)] = a.convert
            else:
                lines.append(fmt_setter(attr_name, arg_name))
        elif has_factory:
            args.append("{arg_name}=NOTHING".format(arg_name=arg_name))
            lines.append("if {arg_name} is not NOTHING:"
                         .format(arg_name=arg_name))
            init_factory_name = _init_factory_pat.format(a.name)
            if a.convert is not None:
                lines.append("    " + fmt_setter_with_converter(attr_name,
                                                                arg_name))
                lines.append("else:")
                lines.append("    " + fmt_setter_with_converter(
                    attr_name,
                    init_factory_name + "({0})".format(maybe_self)
                ))
                names_for_globals[_init_convert_pat.format(a.name)] = a.convert
            else:
                lines.append("    " + fmt_setter(attr_name, arg_name))
                lines.append("else:")
                lines.append("    " + fmt_setter(
                    attr_name,
                    init_factory_name + "({0})".format(maybe_self)
                ))
            names_for_globals[init_factory_name] = a.default.factory
        else:
            args.append(arg_name)
            if a.convert is not None:
                lines.append(fmt_setter_with_converter(attr_name, arg_name))
                names_for_globals[_init_convert_pat.format(a.name)] = a.convert
            else:
                lines.append(fmt_setter(attr_name, arg_name))

    if attrs_to_validate:  # we can skip this if there are no validators.
        names_for_globals["_config"] = _config
        lines.append("if _config._run_validators is True:")
        for a in attrs_to_validate:
            val_name = "__attr_validator_{}".format(a.name)
            attr_name = "__attr_{}".format(a.name)
            lines.append("    {}(self, {}, self.{})".format(
                val_name, attr_name, a.name))
            names_for_globals[val_name] = a.validator
            names_for_globals[attr_name] = a
    if post_init:
        lines.append("self.__attrs_post_init__()")

    return """\
def __init__(self, {args}):
    {lines}
""".format(
        args=", ".join(args),
        lines="\n    ".join(lines) if lines else "pass",
    ), names_for_globals


class Attribute(object):
    """
    *Read-only* representation of an attribute.

    :attribute name: The name of the attribute.

    Plus *all* arguments of :func:`attr.ib`.
    """
    __slots__ = (
        "name", "default", "validator", "repr", "cmp", "hash", "init",
        "convert", "metadata", "type"
    )

    def __init__(self, name, default, validator, repr, cmp, hash, init,
                 convert=None, metadata=None, type=None):
        # Cache this descriptor here to speed things up later.
        bound_setattr = _obj_setattr.__get__(self, Attribute)

        bound_setattr("name", name)
        bound_setattr("default", default)
        bound_setattr("validator", validator)
        bound_setattr("repr", repr)
        bound_setattr("cmp", cmp)
        bound_setattr("hash", hash)
        bound_setattr("init", init)
        bound_setattr("convert", convert)
        bound_setattr("metadata", (metadata_proxy(metadata) if metadata
                                   else _empty_metadata_singleton))
        bound_setattr("type", type)

    def __setattr__(self, name, value):
        raise FrozenInstanceError()

    @classmethod
    def from_counting_attr(cls, name, ca, type=None):
        # type holds the annotated value. deal with conflicts:
        if type is None:
            type = ca.type
        elif ca.type is not None:
            raise ValueError(
                "Type annotation and type argument cannot both be present"
            )
        inst_dict = {
            k: getattr(ca, k)
            for k
            in Attribute.__slots__
            if k not in (
                "name", "validator", "default", "type"
            )  # exclude methods
        }
        return cls(name=name, validator=ca._validator, default=ca._default,
                   type=type, **inst_dict)

    # Don't use _add_pickle since fields(Attribute) doesn't work
    def __getstate__(self):
        """
        Play nice with pickle.
        """
        return tuple(getattr(self, name) if name != "metadata"
                     else dict(self.metadata)
                     for name in self.__slots__)

    def __setstate__(self, state):
        """
        Play nice with pickle.
        """
        bound_setattr = _obj_setattr.__get__(self, Attribute)
        for name, value in zip(self.__slots__, state):
            if name != "metadata":
                bound_setattr(name, value)
            else:
                bound_setattr(name, metadata_proxy(value) if value else
                              _empty_metadata_singleton)


_a = [Attribute(name=name, default=NOTHING, validator=None,
                repr=True, cmp=True, hash=(name != "metadata"), init=True)
      for name in Attribute.__slots__]

Attribute = _add_hash(
    _add_cmp(_add_repr(Attribute, attrs=_a), attrs=_a),
    attrs=[a for a in _a if a.hash]
)


class _CountingAttr(object):
    """
    Intermediate representation of attributes that uses a counter to preserve
    the order in which the attributes have been defined.

    *Internal* data structure of the attrs library.  Running into is most
    likely the result of a bug like a forgotten `@attr.s` decorator.
    """
    __slots__ = ("counter", "_default", "repr", "cmp", "hash", "init",
                 "metadata", "_validator", "convert", "type")
    __attrs_attrs__ = tuple(
        Attribute(name=name, default=NOTHING, validator=None,
                  repr=True, cmp=True, hash=True, init=True)
        for name
        in ("counter", "_default", "repr", "cmp", "hash", "init",)
    ) + (
        Attribute(name="metadata", default=None, validator=None,
                  repr=True, cmp=True, hash=False, init=True),
    )
    cls_counter = 0

    def __init__(self, default, validator, repr, cmp, hash, init, convert,
                 metadata, type):
        _CountingAttr.cls_counter += 1
        self.counter = _CountingAttr.cls_counter
        self._default = default
        # If validator is a list/tuple, wrap it using helper validator.
        if validator and isinstance(validator, (list, tuple)):
            self._validator = and_(*validator)
        else:
            self._validator = validator
        self.repr = repr
        self.cmp = cmp
        self.hash = hash
        self.init = init
        self.convert = convert
        self.metadata = metadata
        self.type = type

    def validator(self, meth):
        """
        Decorator that adds *meth* to the list of validators.

        Returns *meth* unchanged.

        .. versionadded:: 17.1.0
        """
        if self._validator is None:
            self._validator = meth
        else:
            self._validator = and_(self._validator, meth)
        return meth

    def default(self, meth):
        """
        Decorator that allows to set the default for an attribute.

        Returns *meth* unchanged.

        :raises DefaultAlreadySetError: If default has been set before.

        .. versionadded:: 17.1.0
        """
        if self._default is not NOTHING:
            raise DefaultAlreadySetError()

        self._default = Factory(meth, takes_self=True)

        return meth


_CountingAttr = _add_cmp(_add_repr(_CountingAttr))


@attrs(slots=True, init=False, hash=True)
class Factory(object):
    """
    Stores a factory callable.

    If passed as the default value to :func:`attr.ib`, the factory is used to
    generate a new value.

    :param callable factory: A callable that takes either none or exactly one
        mandatory positional argument depending on *takes_self*.
    :param bool takes_self: Pass the partially initialized instance that is
        being initialized as a positional argument.

    .. versionadded:: 17.1.0  *takes_self*
    """
    factory = attrib()
    takes_self = attrib()

    def __init__(self, factory, takes_self=False):
        """
        `Factory` is part of the default machinery so if we want a default
        value here, we have to implement it ourselves.
        """
        self.factory = factory
        self.takes_self = takes_self


def make_class(name, attrs, bases=(object,), **attributes_arguments):
    """
    A quick way to create a new class called *name* with *attrs*.

    :param name: The name for the new class.
    :type name: str

    :param attrs: A list of names or a dictionary of mappings of names to
        attributes.
    :type attrs: :class:`list` or :class:`dict`

    :param tuple bases: Classes that the new class will subclass.

    :param attributes_arguments: Passed unmodified to :func:`attr.s`.

    :return: A new class with *attrs*.
    :rtype: type

    ..  versionadded:: 17.1.0 *bases*
    """
    if isinstance(attrs, dict):
        cls_dict = attrs
    elif isinstance(attrs, (list, tuple)):
        cls_dict = dict((a, attrib()) for a in attrs)
    else:
        raise TypeError("attrs argument must be a dict or a list.")

    post_init = cls_dict.pop("__attrs_post_init__", None)
    type_ = type(
        name,
        bases,
        {} if post_init is None else {"__attrs_post_init__": post_init}
    )
    # For pickling to work, the __module__ variable needs to be set to the
    # frame where the class is created.  Bypass this step in environments where
    # sys._getframe is not defined (Jython for example) or sys._getframe is not
    # defined for arguments greater than 0 (IronPython).
    try:
        type_.__module__ = sys._getframe(1).f_globals.get(
            "__name__", "__main__",
        )
    except (AttributeError, ValueError):
        pass

    return _attrs(these=cls_dict, **attributes_arguments)(type_)


# These are required by within this module so we define them here and merely
# import into .validators.


@attrs(slots=True, hash=True)
class _AndValidator(object):
    """
    Compose many validators to a single one.
    """
    _validators = attrib()

    def __call__(self, inst, attr, value):
        for v in self._validators:
            v(inst, attr, value)


def and_(*validators):
    """
    A validator that composes multiple validators into one.

    When called on a value, it runs all wrapped validators.

    :param validators: Arbitrary number of validators.
    :type validators: callables

    .. versionadded:: 17.1.0
    """
    vals = []
    for validator in validators:
        vals.extend(
            validator._validators if isinstance(validator, _AndValidator)
            else [validator]
        )

    return _AndValidator(tuple(vals))
