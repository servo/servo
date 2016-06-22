# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import re


def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super", "box", "move"]:  # Rust keywords
        name += "_"
    return name


def to_camel_case(ident):
    return re.sub("(^|_|-)([a-z])", lambda m: m.group(2).upper(), ident.strip("_").strip("-"))


class Keyword(object):
    def __init__(self, name, values, gecko_constant_prefix=None,
                 extra_gecko_values=None, extra_servo_values=None):
        self.name = name
        self.values = values.split()
        self.gecko_constant_prefix = gecko_constant_prefix or \
            "NS_STYLE_" + self.name.upper().replace("-", "_")
        self.extra_gecko_values = (extra_gecko_values or "").split()
        self.extra_servo_values = (extra_servo_values or "").split()

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
        return self.gecko_constant_prefix + "_" + value.replace("-moz-", "").replace("-", "_").upper()


class Longhand(object):
    def __init__(self, style_struct, name, animatable=None, derived_from=None, keyword=None,
                 predefined_type=None, custom_cascade=False, experimental=False, internal=False,
                 need_clone=False, need_index=False, gecko_ffi_name=None, depend_on_viewport_size=False):
        self.name = name
        self.keyword = keyword
        self.predefined_type = predefined_type
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.style_struct = style_struct
        self.experimental = ("layout.%s.enabled" % name) if experimental else None
        self.custom_cascade = custom_cascade
        self.internal = internal
        self.need_index = need_index
        self.gecko_ffi_name = gecko_ffi_name or "m" + self.camel_case
        self.depend_on_viewport_size = depend_on_viewport_size
        self.derived_from = (derived_from or "").split()

        # This is done like this since just a plain bool argument seemed like
        # really random.
        if animatable is None:
            raise TypeError("animatable should be specified for " + name + ")")
        if isinstance(animatable, bool):
            self.animatable = animatable
        else:
            assert animatable == "True" or animatable == "False"
            self.animatable = animatable == "True"

        # NB: Animatable implies clone because a property animation requires a
        # copy of the computed value.
        #
        # See components/style/helpers/animated_properties.mako.rs.
        self.need_clone = need_clone or self.animatable


class Shorthand(object):
    def __init__(self, name, sub_properties, experimental=False, internal=False):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.derived_from = None
        self.experimental = ("layout.%s.enabled" % name) if experimental else None
        self.sub_properties = sub_properties
        self.internal = internal


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
        self.servo_struct_name = "Servo" + name
        self.gecko_struct_name = "Gecko" + name
        self.trait_name = name
        self.trait_name_lower = name.lower()
        self.ident = to_rust_ident(self.trait_name_lower)
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
        self.derived_longhands = {}
        self.shorthands = []

    def new_style_struct(self, *args, **kwargs):
        style_struct = StyleStruct(*args, **kwargs)
        self.style_structs.append(style_struct)
        self.current_style_struct = style_struct

    def active_style_structs(self):
        return [s for s in self.style_structs if s.additional_methods or s.longhands]

    def declare_longhand(self, name, products="gecko servo", **kwargs):
        products = products.split()
        if self.product not in products:
            return

        longand = Longhand(self.current_style_struct, name, **kwargs)
        self.current_style_struct.longhands.append(longand)
        self.longhands.append(longand)
        self.longhands_by_name[name] = longand

        for name in longand.derived_from:
            self.derived_longhands.setdefault(name, []).append(longand)

        return longand

    def declare_shorthand(self, name, sub_properties, products="gecko servo", *args, **kwargs):
        products = products.split()
        if self.product not in products:
            return

        sub_properties = [self.longhands_by_name[s] for s in sub_properties]
        shorthand = Shorthand(name, sub_properties, *args, **kwargs)
        self.shorthands.append(shorthand)
        return shorthand
