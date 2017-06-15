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

SYSTEM_FONT_LONGHANDS = """font_family font_size font_style
                           font_variant_caps font_stretch font_kerning
                           font_variant_position font_weight
                           font_size_adjust font_variant_alternates
                           font_variant_ligatures font_variant_east_asian
                           font_variant_numeric font_language_override
                           font_feature_settings""".split()


def maybe_moz_logical_alias(product, side, prop):
    if product == "gecko" and side[1]:
        axis, dir = side[0].split("-")
        if axis == "inline":
            return prop % dir
    return None


def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super", "box", "move"]:  # Rust keywords
        name += "_"
    return name


def to_camel_case(ident):
    return re.sub("(^|_|-)([a-z])", lambda m: m.group(2).upper(), ident.strip("_").strip("-"))


def to_camel_case_lower(ident):
    camel = to_camel_case(ident)
    return camel[0].lower() + camel[1:]


def parse_aliases(value):
    aliases = {}
    for pair in value.split():
        [a, v] = pair.split("=")
        aliases[a] = v
    return aliases


class Keyword(object):
    def __init__(self, name, values, gecko_constant_prefix=None,
                 gecko_enum_prefix=None, custom_consts=None,
                 extra_gecko_values=None, extra_servo_values=None,
                 aliases=None,
                 extra_gecko_aliases=None, extra_servo_aliases=None,
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
        self.aliases = parse_aliases(aliases or "")
        self.extra_gecko_aliases = parse_aliases(extra_gecko_aliases or "")
        self.extra_servo_aliases = parse_aliases(extra_servo_aliases or "")
        self.consts_map = {} if custom_consts is None else custom_consts
        self.gecko_strip_moz_prefix = gecko_strip_moz_prefix
        self.gecko_inexhaustive = gecko_inexhaustive or (gecko_enum_prefix is None)

    def gecko_values(self):
        return self.values + self.extra_gecko_values

    def servo_values(self):
        return self.values + self.extra_servo_values

    def gecko_aliases(self):
        aliases = self.aliases.copy()
        aliases.update(self.extra_gecko_aliases)
        return aliases

    def servo_aliases(self):
        aliases = self.aliases.copy()
        aliases.update(self.extra_servo_aliases)
        return aliases

    def values_for(self, product):
        if product == "gecko":
            return self.gecko_values()
        elif product == "servo":
            return self.servo_values()
        else:
            raise Exception("Bad product: " + product)

    def aliases_for(self, product):
        if product == "gecko":
            return self.gecko_aliases()
        elif product == "servo":
            return self.servo_aliases()
        else:
            raise Exception("Bad product: " + product)

    def gecko_constant(self, value):
        moz_stripped = value.replace("-moz-", '') if self.gecko_strip_moz_prefix else value.replace("-moz-", 'moz-')
        mapped = self.consts_map.get(value)
        if self.gecko_enum_prefix:
            parts = moz_stripped.replace('-', '_').split('_')
            parts = mapped if mapped else [p.title() for p in parts]
            return self.gecko_enum_prefix + "::" + "".join(parts)
        else:
            suffix = mapped if mapped else moz_stripped.replace("-", "_")
            return self.gecko_constant_prefix + "_" + suffix.upper()

    def needs_cast(self):
        return self.gecko_enum_prefix is None

    def maybe_cast(self, type_str):
        return "as " + type_str if self.needs_cast() else ""

    def casted_constant_name(self, value, cast_type):
        if cast_type is None:
            raise TypeError("We should specify the cast_type.")

        if self.gecko_enum_prefix is None:
            return cast_type.upper() + "_" + self.gecko_constant(value)
        else:
            return cast_type.upper() + "_" + self.gecko_constant(value).upper().replace("::", "_")


def arg_to_bool(arg):
    if isinstance(arg, bool):
        return arg
    assert arg in ["True", "False"]
    return arg == "True"


class Longhand(object):
    def __init__(self, style_struct, name, spec=None, animation_value_type=None, derived_from=None, keyword=None,
                 predefined_type=None, custom_cascade=False, experimental=False, internal=False,
                 need_clone=False, need_index=False, gecko_ffi_name=None, depend_on_viewport_size=False,
                 allowed_in_keyframe_block=True, cast_type='u8',
                 has_uncacheable_values=False, logical=False, alias=None, extra_prefixes=None, boxed=False,
                 flags=None, allowed_in_page_rule=False, allow_quirks=False, ignored_when_colors_disabled=False,
                 vector=False):
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
        self.cast_type = cast_type
        self.logical = arg_to_bool(logical)
        self.alias = alias.split() if alias else []
        self.extra_prefixes = extra_prefixes.split() if extra_prefixes else []
        self.boxed = arg_to_bool(boxed)
        self.flags = flags.split() if flags else []
        self.allowed_in_page_rule = arg_to_bool(allowed_in_page_rule)
        self.allow_quirks = allow_quirks
        self.ignored_when_colors_disabled = ignored_when_colors_disabled
        self.is_vector = vector

        # https://drafts.csswg.org/css-animations/#keyframes
        # > The <declaration-list> inside of <keyframe-block> accepts any CSS property
        # > except those defined in this specification,
        # > but does accept the `animation-play-state` property and interprets it specially.
        self.allowed_in_keyframe_block = allowed_in_keyframe_block \
            and allowed_in_keyframe_block != "False"

        # This is done like this since just a plain bool argument seemed like
        # really random.
        if animation_value_type is None:
            raise TypeError("animation_value_type should be specified for (" + name + ")")
        self.animation_value_type = animation_value_type

        self.animatable = animation_value_type != "none"
        self.transitionable = animation_value_type != "none" \
            and animation_value_type != "discrete"
        self.is_animatable_with_computed_value = animation_value_type == "ComputedValue" \
            or animation_value_type == "discrete"
        if self.logical:
            # Logical properties will be animatable (i.e. the animation type is
            # discrete). For now, it is still non-animatable.
            self.animatable = False
            self.transitionable = False
            self.animation_type = None
        # NB: Animatable implies clone because a property animation requires a
        # copy of the computed value.
        #
        # See components/style/helpers/animated_properties.mako.rs.
        self.need_clone = need_clone or self.animatable


class Shorthand(object):
    def __init__(self, name, sub_properties, spec=None, experimental=False, internal=False,
                 allowed_in_keyframe_block=True, alias=None, extra_prefixes=None,
                 allowed_in_page_rule=False, flags=None):
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
        self.alias = alias.split() if alias else []
        self.extra_prefixes = extra_prefixes.split() if extra_prefixes else []
        self.allowed_in_page_rule = arg_to_bool(allowed_in_page_rule)
        self.flags = flags.split() if flags else []

        # https://drafts.csswg.org/css-animations/#keyframes
        # > The <declaration-list> inside of <keyframe-block> accepts any CSS property
        # > except those defined in this specification,
        # > but does accept the `animation-play-state` property and interprets it specially.
        self.allowed_in_keyframe_block = allowed_in_keyframe_block \
            and allowed_in_keyframe_block != "False"

    def get_animatable(self):
        animatable = False
        for sub in self.sub_properties:
            if sub.animatable:
                animatable = True
                break
        return animatable

    def get_transitionable(self):
        transitionable = False
        for sub in self.sub_properties:
            if sub.transitionable:
                transitionable = True
                break
        return transitionable

    animatable = property(get_animatable)
    transitionable = property(get_transitionable)


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
        which code exists for either servo or stylo are generated. Note that we skip
        this behavior when the style crate is being built in gecko mode, because we
        need manual glue for such properties and we don't have it.
    """
    def __init__(self, product, testing):
        self.product = product
        self.testing = testing and product != "gecko"
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

    def add_prefixed_aliases(self, property):
        # FIXME Servo's DOM architecture doesn't support vendor-prefixed properties.
        #       See servo/servo#14941.
        if self.product == "gecko":
            for prefix in property.extra_prefixes:
                property.alias.append('-%s-%s' % (prefix, property.name))

    def declare_longhand(self, name, products="gecko servo", disable_when_testing=False, **kwargs):
        products = products.split()
        if self.product not in products and not (self.testing and not disable_when_testing):
            return

        longhand = Longhand(self.current_style_struct, name, **kwargs)
        self.add_prefixed_aliases(longhand)
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
        self.add_prefixed_aliases(shorthand)
        self.shorthands.append(shorthand)
        return shorthand

    def shorthands_except_all(self):
        return [s for s in self.shorthands if s.name != "all"]
