# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import re

PHYSICAL_SIDES = ["top", "left", "bottom", "right"]
LOGICAL_SIDES = ["block-start", "block-end", "inline-start", "inline-end"]
PHYSICAL_SIZES = ["width", "height"]
LOGICAL_SIZES = ["block-size", "inline-size"]

# bool is True when logical
ALL_SIDES = [(side, False) for side in PHYSICAL_SIDES] + [(side, True) for side in LOGICAL_SIDES]
ALL_SIZES = [(size, False) for size in PHYSICAL_SIZES] + [(size, True) for size in LOGICAL_SIZES]


def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super", "box", "move"]:  # Rust keywords
        name += "_"
    return name


def to_camel_case(ident):
    return re.sub("(^|_|-)([a-z])", lambda m: m.group(2).upper(), ident.strip("_").strip("-"))


class Keyword(object):
    def __init__(self, name, values, gecko_constant_prefix=None,
                 gecko_enum_prefix=None, custom_consts=None,
                 extra_gecko_values=None, extra_servo_values=None,
                 gecko_strip_moz_prefix=True,
                 gecko_inexhaustive=None):
        self.name = name
        self.values = values.split()
        if gecko_constant_prefix and gecko_enum_prefix:
            raise TypeError("Only one of gecko_constant_prefix and gecko_enum_prefix can be specified")
        self.gecko_constant_prefix = gecko_constant_prefix or \
            "NS_STYLE_" + self.name.upper().replace("-", "_")
        self.gecko_enum_prefix = gecko_enum_prefix
        self.extra_gecko_values = (extra_gecko_values or "").split()
        self.extra_servo_values = (extra_servo_values or "").split()
        self.consts_map = {} if custom_consts is None else custom_consts
        self.gecko_strip_moz_prefix = gecko_strip_moz_prefix
        self.gecko_inexhaustive = gecko_inexhaustive or (gecko_enum_prefix is None)

    def gecko_values(self):
        return self.values + self.extra_gecko_values

    def servo_values(self):
        return self.values + self.extra_servo_values

    def values_for(self, product):
        if product == "gecko":
            return self.gecko_values()
        elif product == "servo":
            return self.servo_values()
        else:
            raise Exception("Bad product: " + product)

    def gecko_constant(self, value):
        moz_stripped = value.replace("-moz-", '') if self.gecko_strip_moz_prefix else value
        parts = moz_stripped.split('-')
        if self.gecko_enum_prefix:
            parts = [p.title() for p in parts]
            return self.gecko_enum_prefix + "::" + "".join(parts)
        else:
            mapped = self.consts_map.get(value)
            suffix = mapped if mapped else moz_stripped.replace("-", "_")
            return self.gecko_constant_prefix + "_" + suffix.upper()

    def needs_cast(self):
        return self.gecko_enum_prefix is None

    def maybe_cast(self, type_str):
        return "as " + type_str if self.needs_cast() else ""


def arg_to_bool(arg):
    if isinstance(arg, bool):
        return arg
    assert arg in ["True", "False"]
    return arg == "True"


class Longhand(object):
    def __init__(self, style_struct, name, spec=None, animatable=None, derived_from=None, keyword=None,
                 predefined_type=None, custom_cascade=False, experimental=False, internal=False,
                 need_clone=False, need_index=False, gecko_ffi_name=None, depend_on_viewport_size=False,
                 allowed_in_keyframe_block=True, complex_color=False, cast_type='u8',
                 has_uncacheable_values=False, logical=False):
        self.name = name
        if not spec:
            raise TypeError("Spec should be specified for %s" % name)
        self.spec = spec
        self.keyword = keyword
        self.predefined_type = predefined_type
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.style_struct = style_struct
        self.experimental = ("layout.%s.enabled" % name) if experimental else None
        self.custom_cascade = custom_cascade
        self.internal = internal
        self.need_index = need_index
        self.has_uncacheable_values = has_uncacheable_values
        self.gecko_ffi_name = gecko_ffi_name or "m" + self.camel_case
        self.depend_on_viewport_size = depend_on_viewport_size
        self.derived_from = (derived_from or "").split()
        self.complex_color = complex_color
        self.cast_type = cast_type
        self.logical = arg_to_bool(logical)

        # https://drafts.csswg.org/css-animations/#keyframes
        # > The <declaration-list> inside of <keyframe-block> accepts any CSS property
        # > except those defined in this specification,
        # > but does accept the `animation-play-state` property and interprets it specially.
        self.allowed_in_keyframe_block = allowed_in_keyframe_block \
            and allowed_in_keyframe_block != "False"

        # This is done like this since just a plain bool argument seemed like
        # really random.
        if animatable is None:
            raise TypeError("animatable should be specified for " + name + ")")
        self.animatable = arg_to_bool(animatable)
        if self.logical:
            # Logical properties don't animate separately
            self.animatable = False
        # NB: Animatable implies clone because a property animation requires a
        # copy of the computed value.
        #
        # See components/style/helpers/animated_properties.mako.rs.
        self.need_clone = need_clone or self.animatable


class Shorthand(object):
    def __init__(self, name, sub_properties, spec=None, experimental=False, internal=False,
                 allowed_in_keyframe_block=True):
        self.name = name
        if not spec:
            raise TypeError("Spec should be specified for %s" % name)
        self.spec = spec
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.derived_from = None
        self.experimental = ("layout.%s.enabled" % name) if experimental else None
        self.sub_properties = sub_properties
        self.internal = internal

        # https://drafts.csswg.org/css-animations/#keyframes
        # > The <declaration-list> inside of <keyframe-block> accepts any CSS property
        # > except those defined in this specification,
        # > but does accept the `animation-play-state` property and interprets it specially.
        self.allowed_in_keyframe_block = allowed_in_keyframe_block \
            and allowed_in_keyframe_block != "False"


class Method(object):
    def __init__(self, name, return_type=None, arg_types=None, is_mut=False):
        self.name = name
        self.return_type = return_type
        self.arg_types = arg_types or []
        self.is_mut = is_mut

    def arg_list(self):
        args = ["_: " + x for x in self.arg_types]
        args = ["&mut self" if self.is_mut else "&self"] + args
        return ", ".join(args)

    def signature(self):
        sig = "fn %s(%s)" % (self.name, self.arg_list())
        if self.return_type:
            sig = sig + " -> " + self.return_type
        return sig

    def declare(self):
        return self.signature() + ";"

    def stub(self):
        return self.signature() + "{ unimplemented!() }"


class StyleStruct(object):
    def __init__(self, name, inherited, gecko_name=None, additional_methods=None):
        self.gecko_struct_name = "Gecko" + name
        self.name = name
        self.name_lower = name.lower()
        self.ident = to_rust_ident(self.name_lower)
        self.longhands = []
        self.inherited = inherited
        self.gecko_name = gecko_name or name
        self.gecko_ffi_name = "nsStyle" + self.gecko_name
        self.additional_methods = additional_methods or []


class PropertiesData(object):
    """
        The `testing` parameter means that we're running tests.

        In this situation, the `product` value is ignored while choosing
        which shorthands and longhands to generate; and instead all properties for
        which code exists for either servo or stylo are generated.
    """
    def __init__(self, product, testing):
        self.product = product
        self.testing = testing
        self.style_structs = []
        self.current_style_struct = None
        self.longhands = []
        self.longhands_by_name = {}
        self.derived_longhands = {}
        self.shorthands = []

    def new_style_struct(self, *args, **kwargs):
        style_struct = StyleStruct(*args, **kwargs)
        self.style_structs.append(style_struct)
        self.current_style_struct = style_struct

    def active_style_structs(self):
        return [s for s in self.style_structs if s.additional_methods or s.longhands]

    def declare_longhand(self, name, products="gecko servo", disable_when_testing=False, **kwargs):
        products = products.split()
        if self.product not in products and not (self.testing and not disable_when_testing):
            return

        longhand = Longhand(self.current_style_struct, name, **kwargs)
        self.current_style_struct.longhands.append(longhand)
        self.longhands.append(longhand)
        self.longhands_by_name[name] = longhand

        for name in longhand.derived_from:
            self.derived_longhands.setdefault(name, []).append(longhand)

        return longhand

    def declare_shorthand(self, name, sub_properties, products="gecko servo",
                          disable_when_testing=False, *args, **kwargs):
        products = products.split()
        if self.product not in products and not (self.testing and not disable_when_testing):
            return

        sub_properties = [self.longhands_by_name[s] for s in sub_properties]
        shorthand = Shorthand(name, sub_properties, *args, **kwargs)
        self.shorthands.append(shorthand)
        return shorthand
