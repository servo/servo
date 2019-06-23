# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import re

PHYSICAL_SIDES = ["top", "right", "bottom", "left"]
LOGICAL_SIDES = ["block-start", "block-end", "inline-start", "inline-end"]
PHYSICAL_SIZES = ["width", "height"]
LOGICAL_SIZES = ["block-size", "inline-size"]
PHYSICAL_CORNERS = ["top-left", "top-right", "bottom-right", "bottom-left"]
LOGICAL_CORNERS = ["start-start", "start-end", "end-start", "end-end"]
PHYSICAL_AXES = ["x", "y"]
LOGICAL_AXES = ["inline", "block"]

# bool is True when logical
ALL_SIDES = [(side, False) for side in PHYSICAL_SIDES] + [(side, True) for side in LOGICAL_SIDES]
ALL_SIZES = [(size, False) for size in PHYSICAL_SIZES] + [(size, True) for size in LOGICAL_SIZES]
ALL_CORNERS = [(corner, False) for corner in PHYSICAL_CORNERS] + [(corner, True) for corner in LOGICAL_CORNERS]
ALL_AXES = [(axis, False) for axis in PHYSICAL_AXES] + [(axis, True) for axis in LOGICAL_AXES]

SYSTEM_FONT_LONGHANDS = """font_family font_size font_style
                           font_variant_caps font_stretch font_kerning
                           font_variant_position font_weight
                           font_size_adjust font_variant_alternates
                           font_variant_ligatures font_variant_east_asian
                           font_variant_numeric font_language_override
                           font_feature_settings font_variation_settings
                           font_optical_sizing""".split()


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


def to_snake_case(ident):
    return re.sub("([A-Z]+)", lambda m: "_" + m.group(1).lower(), ident).strip("_")


def to_camel_case(ident):
    return re.sub("(^|_|-)([a-z0-9])", lambda m: m.group(2).upper(), ident.strip("_").strip("-"))


def to_camel_case_lower(ident):
    camel = to_camel_case(ident)
    return camel[0].lower() + camel[1:]


# https://drafts.csswg.org/cssom/#css-property-to-idl-attribute
def to_idl_name(ident):
    return re.sub("-([a-z])", lambda m: m.group(1).upper(), ident)


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
                 extra_gecko_aliases=None,
                 gecko_strip_moz_prefix=None,
                 gecko_inexhaustive=None):
        self.name = name
        self.values = values.split()
        if gecko_constant_prefix and gecko_enum_prefix:
            raise TypeError("Only one of gecko_constant_prefix and gecko_enum_prefix "
                            "can be specified")
        self.gecko_constant_prefix = gecko_constant_prefix or \
            "NS_STYLE_" + self.name.upper().replace("-", "_")
        self.gecko_enum_prefix = gecko_enum_prefix
        self.extra_gecko_values = (extra_gecko_values or "").split()
        self.extra_servo_values = (extra_servo_values or "").split()
        self.aliases = parse_aliases(aliases or "")
        self.extra_gecko_aliases = parse_aliases(extra_gecko_aliases or "")
        self.consts_map = {} if custom_consts is None else custom_consts
        self.gecko_strip_moz_prefix = True \
            if gecko_strip_moz_prefix is None else gecko_strip_moz_prefix
        self.gecko_inexhaustive = gecko_inexhaustive or (gecko_enum_prefix is None)

    def gecko_values(self):
        return self.values + self.extra_gecko_values

    def servo_values(self):
        return self.values + self.extra_servo_values

    def gecko_aliases(self):
        aliases = self.aliases.copy()
        aliases.update(self.extra_gecko_aliases)
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
            return self.aliases
        else:
            raise Exception("Bad product: " + product)

    def gecko_constant(self, value):
        moz_stripped = (value.replace("-moz-", '')
                        if self.gecko_strip_moz_prefix else value.replace("-moz-", 'moz-'))
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
    assert arg in ["True", "False"], "Unexpected value for boolean arguement: " + repr(arg)
    return arg == "True"


def parse_property_aliases(alias_list):
    result = []
    if alias_list:
        for alias in alias_list.split():
            (name, _, pref) = alias.partition(":")
            result.append((name, pref))
    return result


def to_phys(name, logical, physical):
    return name.replace(logical, physical).replace("inset-", "")


class Longhand(object):
    def __init__(self, style_struct, name, spec=None, animation_value_type=None, keyword=None,
                 predefined_type=None, servo_pref=None, gecko_pref=None,
                 enabled_in="content", need_index=False,
                 gecko_ffi_name=None,
                 allowed_in_keyframe_block=True, cast_type='u8',
                 logical=False, logical_group=None, alias=None, extra_prefixes=None, boxed=False,
                 flags=None, allowed_in_page_rule=False, allow_quirks="No",
                 ignored_when_colors_disabled=False,
                 simple_vector_bindings=False,
                 vector=False, servo_restyle_damage="repaint"):
        self.name = name
        if not spec:
            raise TypeError("Spec should be specified for %s" % name)
        self.spec = spec
        self.keyword = keyword
        self.predefined_type = predefined_type
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.style_struct = style_struct
        self.servo_pref = servo_pref
        self.gecko_pref = gecko_pref
        # For enabled_in, the setup is as follows:
        # It needs to be one of the four values: ["", "ua", "chrome", "content"]
        #  * "chrome" implies "ua", and implies that they're explicitly
        #    enabled.
        #  * "" implies the property will never be parsed.
        #  * "content" implies the property is accessible unconditionally,
        #    modulo a pref, set via servo_pref / gecko_pref.
        assert enabled_in in ["", "ua", "chrome", "content"]
        self.enabled_in = enabled_in
        self.need_index = need_index
        self.gecko_ffi_name = gecko_ffi_name or "m" + self.camel_case
        self.cast_type = cast_type
        self.logical = arg_to_bool(logical)
        self.logical_group = logical_group
        if self.logical:
            assert logical_group, "Property " + name + " must have a logical group"
        self.alias = parse_property_aliases(alias)
        self.extra_prefixes = parse_property_aliases(extra_prefixes)
        self.boxed = arg_to_bool(boxed)
        self.flags = flags.split() if flags else []
        self.allowed_in_page_rule = arg_to_bool(allowed_in_page_rule)
        self.allow_quirks = allow_quirks
        self.ignored_when_colors_disabled = ignored_when_colors_disabled
        self.is_vector = vector
        self.simple_vector_bindings = simple_vector_bindings

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

        # See compute_damage for the various values this can take
        self.servo_restyle_damage = servo_restyle_damage

    @staticmethod
    def type():
        return "longhand"

    # For a given logical property return all the physical
    # property names corresponding to it.
    def all_physical_mapped_properties(self):
        assert self.logical
        candidates = [s for s in LOGICAL_SIDES + LOGICAL_SIZES + LOGICAL_CORNERS
                      if s in self.name] + [s for s in LOGICAL_AXES if self.name.endswith(s)]
        assert(len(candidates) == 1)
        logical_side = candidates[0]

        physical = PHYSICAL_SIDES if logical_side in LOGICAL_SIDES \
            else PHYSICAL_SIZES if logical_side in LOGICAL_SIZES \
            else PHYSICAL_AXES if logical_side in LOGICAL_AXES \
            else LOGICAL_CORNERS
        return [to_phys(self.name, logical_side, physical_side)
                for physical_side in physical]

    def experimental(self, product):
        if product == "gecko":
            return bool(self.gecko_pref)
        return bool(self.servo_pref)

    # FIXME(emilio): Shorthand and Longhand should really share a base class.
    def explicitly_enabled_in_ua_sheets(self):
        return self.enabled_in in ["ua", "chrome"]

    def explicitly_enabled_in_chrome(self):
        return self.enabled_in == "chrome"

    def enabled_in_content(self):
        return self.enabled_in == "content"

    def may_be_disabled_in(self, shorthand, product):
        if product == "gecko":
            return self.gecko_pref and self.gecko_pref != shorthand.gecko_pref
        return self.servo_pref and self.servo_pref != shorthand.servo_pref

    def base_type(self):
        if self.predefined_type and not self.is_vector:
            return "crate::values::specified::{}".format(self.predefined_type)
        return "longhands::{}::SpecifiedValue".format(self.ident)

    def specified_type(self):
        if self.predefined_type and not self.is_vector:
            ty = "crate::values::specified::{}".format(self.predefined_type)
        else:
            ty = "longhands::{}::SpecifiedValue".format(self.ident)
        if self.boxed:
            ty = "Box<{}>".format(ty)
        return ty

    def specified_is_copy(self):
        if self.is_vector or self.boxed:
            return False
        if self.predefined_type:
            return self.predefined_type in {
                "AlignContent",
                "AlignItems",
                "AlignSelf",
                "Appearance",
                "BreakBetween",
                "BreakWithin",
                "BackgroundRepeat",
                "BorderImageRepeat",
                "BorderStyle",
                "Clear",
                "ColumnCount",
                "Contain",
                "Display",
                "FillRule",
                "Float",
                "FontSizeAdjust",
                "FontStretch",
                "FontStyle",
                "FontStyleAdjust",
                "FontSynthesis",
                "FontVariantEastAsian",
                "FontVariantLigatures",
                "FontVariantNumeric",
                "FontWeight",
                "GreaterThanOrEqualToOneNumber",
                "GridAutoFlow",
                "InitialLetter",
                "Integer",
                "JustifyContent",
                "JustifyItems",
                "JustifySelf",
                "LineBreak",
                "MozForceBrokenImageIcon",
                "MozListReversed",
                "MozScriptLevel",
                "MozScriptMinSize",
                "MozScriptSizeMultiplier",
                "NonNegativeNumber",
                "OffsetRotate",
                "Opacity",
                "OutlineStyle",
                "Overflow",
                "OverflowAnchor",
                "OverflowClipBox",
                "OverflowWrap",
                "OverscrollBehavior",
                "Percentage",
                "PositiveIntegerOrNone",
                "Resize",
                "SVGOpacity",
                "SVGPaintOrder",
                "ScrollSnapAlign",
                "ScrollSnapAxis",
                "ScrollSnapStrictness",
                "ScrollSnapType",
                "TextAlign",
                "TextDecorationLine",
                "TextEmphasisPosition",
                "TextTransform",
                "TouchAction",
                "TransformStyle",
                "UserSelect",
                "WordBreak",
                "XSpan",
                "XTextZoom",
                "ZIndex",
            }
        if self.name == "overflow-y":
            return True
        return bool(self.keyword)

    def animated_type(self):
        assert self.animatable
        computed = "<{} as ToComputedValue>::ComputedValue".format(self.base_type())
        if self.is_animatable_with_computed_value:
            return computed
        return "<{} as ToAnimatedValue>::AnimatedValue".format(computed)

    def nscsspropertyid(self):
        return "nsCSSPropertyID::eCSSProperty_%s" % self.ident


class Shorthand(object):
    def __init__(self, name, sub_properties, spec=None, servo_pref=None, gecko_pref=None,
                 enabled_in="content",
                 allowed_in_keyframe_block=True, alias=None, extra_prefixes=None,
                 allowed_in_page_rule=False, flags=None):
        self.name = name
        if not spec:
            raise TypeError("Spec should be specified for %s" % name)
        self.spec = spec
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.servo_pref = servo_pref
        self.gecko_pref = gecko_pref
        self.sub_properties = sub_properties
        assert enabled_in in ["", "ua", "chrome", "content"]
        self.enabled_in = enabled_in
        self.alias = parse_property_aliases(alias)
        self.extra_prefixes = parse_property_aliases(extra_prefixes)
        self.allowed_in_page_rule = arg_to_bool(allowed_in_page_rule)
        self.flags = flags.split() if flags else []

        # https://drafts.csswg.org/css-animations/#keyframes
        # > The <declaration-list> inside of <keyframe-block> accepts any CSS property
        # > except those defined in this specification,
        # > but does accept the `animation-play-state` property and interprets it specially.
        self.allowed_in_keyframe_block = allowed_in_keyframe_block \
            and allowed_in_keyframe_block != "False"

    def get_animatable(self):
        for sub in self.sub_properties:
            if sub.animatable:
                return True
        return False

    def get_transitionable(self):
        transitionable = False
        for sub in self.sub_properties:
            if sub.transitionable:
                transitionable = True
                break
        return transitionable

    animatable = property(get_animatable)
    transitionable = property(get_transitionable)

    @staticmethod
    def type():
        return "shorthand"

    def experimental(self, product):
        if product == "gecko":
            return bool(self.gecko_pref)
        return bool(self.servo_pref)

    # FIXME(emilio): Shorthand and Longhand should really share a base class.
    def explicitly_enabled_in_ua_sheets(self):
        return self.enabled_in in ["ua", "chrome"]

    def explicitly_enabled_in_chrome(self):
        return self.enabled_in == "chrome"

    def enabled_in_content(self):
        return self.enabled_in == "content"

    def nscsspropertyid(self):
        return "nsCSSPropertyID::eCSSProperty_%s" % self.ident


class Alias(object):
    def __init__(self, name, original, gecko_pref):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.original = original
        self.enabled_in = original.enabled_in
        self.servo_pref = original.servo_pref
        self.animatable = original.animatable
        self.transitionable = original.transitionable
        self.gecko_pref = gecko_pref
        self.allowed_in_page_rule = original.allowed_in_page_rule
        self.allowed_in_keyframe_block = original.allowed_in_keyframe_block

    @staticmethod
    def type():
        return "alias"

    def experimental(self, product):
        if product == "gecko":
            return bool(self.gecko_pref)
        return bool(self.servo_pref)

    def explicitly_enabled_in_ua_sheets(self):
        return self.enabled_in in ["ua", "chrome"]

    def explicitly_enabled_in_chrome(self):
        return self.enabled_in == "chrome"

    def enabled_in_content(self):
        return self.enabled_in == "content"

    def nscsspropertyid(self):
        return "nsCSSPropertyID::eCSSPropertyAlias_%s" % self.ident


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
        self.name_lower = to_snake_case(name)
        self.ident = to_rust_ident(self.name_lower)
        self.longhands = []
        self.inherited = inherited
        self.gecko_name = gecko_name or name
        self.gecko_ffi_name = "nsStyle" + self.gecko_name
        self.additional_methods = additional_methods or []


class PropertiesData(object):
    def __init__(self, product):
        self.product = product
        self.style_structs = []
        self.current_style_struct = None
        self.longhands = []
        self.longhands_by_name = {}
        self.longhand_aliases = []
        self.shorthands = []
        self.shorthand_aliases = []

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
            for (prefix, pref) in property.extra_prefixes:
                property.alias.append(('-%s-%s' % (prefix, property.name), pref))

    def declare_longhand(self, name, products="gecko servo", **kwargs):
        products = products.split()
        if self.product not in products:
            return

        longhand = Longhand(self.current_style_struct, name, **kwargs)
        self.add_prefixed_aliases(longhand)
        longhand.alias = list(map(lambda (x, p): Alias(x, longhand, p), longhand.alias))
        self.longhand_aliases += longhand.alias
        self.current_style_struct.longhands.append(longhand)
        self.longhands.append(longhand)
        self.longhands_by_name[name] = longhand

        return longhand

    def declare_shorthand(self, name, sub_properties, products="gecko servo", *args, **kwargs):
        products = products.split()
        if self.product not in products:
            return

        sub_properties = [self.longhands_by_name[s] for s in sub_properties]
        shorthand = Shorthand(name, sub_properties, *args, **kwargs)
        self.add_prefixed_aliases(shorthand)
        shorthand.alias = list(map(lambda (x, p): Alias(x, shorthand, p), shorthand.alias))
        self.shorthand_aliases += shorthand.alias
        self.shorthands.append(shorthand)
        return shorthand

    def shorthands_except_all(self):
        return [s for s in self.shorthands if s.name != "all"]

    def all_aliases(self):
        return self.longhand_aliases + self.shorthand_aliases
