# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys
import json
import re

SCRIPT_PATH = os.path.abspath(os.path.dirname(__file__))
SERVO_ROOT = os.path.abspath(os.path.join(SCRIPT_PATH, "..", "..", "..", "..", ".."))

FILTER_PATTERN = re.compile("// skip-unless ([A-Z_]+)\n")


def main():
    os.chdir(os.path.join(os.path.dirname(__file__)))
    sys.path.insert(0, os.path.join(SERVO_ROOT, "third_party", "WebIDL"))
    sys.path.insert(0, os.path.join(SERVO_ROOT, "third_party", "ply"))

    css_properties_json, out_dir = sys.argv[1:]
    # Four dotdots: /path/to/target(4)/debug(3)/build(2)/style-*(1)/out
    # Do not ascend above the target dir, because it may not be called target
    # or even have a parent (see CARGO_TARGET_DIR).
    doc_servo = os.path.join(out_dir, "..", "..", "..", "..", "doc")
    webidls_dir = os.path.join(SCRIPT_PATH, "..", "..", "webidls")
    config_file = "Bindings.conf"

    import WebIDL
    from Configuration import Configuration
    from CodegenRust import CGBindingRoot

    parser = WebIDL.Parser(make_dir(os.path.join(out_dir, "cache")), use_builtin_readable_stream=False)
    webidls = [name for name in os.listdir(webidls_dir) if name.endswith(".webidl")]
    for webidl in webidls:
        filename = os.path.join(webidls_dir, webidl)
        with open(filename, "r", encoding="utf-8") as f:
            contents = f.read()
            filter_match = FILTER_PATTERN.search(contents)
            if filter_match:
                env_var = filter_match.group(1)
                if not os.environ.get(env_var):
                    continue

            parser.parse(contents, filename)

    add_css_properties_attributes(css_properties_json, parser)
    parser_results = parser.finish()
    config = Configuration(config_file, parser_results)
    make_dir(os.path.join(out_dir, "Bindings"))

    for name, filename in [
        ("PrototypeList", "PrototypeList.rs"),
        ("RegisterBindings", "RegisterBindings.rs"),
        ("InterfaceObjectMap", "InterfaceObjectMap.rs"),
        ("InterfaceObjectMapData", "InterfaceObjectMapData.json"),
        ("InterfaceTypes", "InterfaceTypes.rs"),
        ("InheritTypes", "InheritTypes.rs"),
        ("Bindings", "Bindings/mod.rs"),
        ("UnionTypes", "UnionTypes.rs"),
        ("DomTypes", "DomTypes.rs"),
        ("DomTypeHolder", "DomTypeHolder.rs"),
    ]:
        generate(config, name, os.path.join(out_dir, filename))
    make_dir(doc_servo)
    generate(config, "SupportedDomApis", os.path.join(doc_servo, "apis.html"))

    for webidl in webidls:
        filename = os.path.join(webidls_dir, webidl)
        prefix = "Bindings/%sBinding" % webidl[:-len(".webidl")]
        module = CGBindingRoot(config, prefix, filename).define()
        if module:
            with open(os.path.join(out_dir, prefix + ".rs"), "wb") as f:
                f.write(module.encode("utf-8"))


def make_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)
    return path


def generate(config, name, filename):
    from CodegenRust import GlobalGenRoots
    root = getattr(GlobalGenRoots, name)(config)
    code = root.define()
    with open(filename, "wb") as f:
        f.write(code.encode("utf-8"))


def add_css_properties_attributes(css_properties_json, parser):
    css_properties = json.load(open(css_properties_json, "rb"))
    idl = "partial interface CSSStyleDeclaration {\n%s\n};\n" % "\n".join(
        "  [%sCEReactions, SetterThrows] attribute [LegacyNullToEmptyString] DOMString %s;" % (
            ('Pref="%s", ' % data["pref"] if data["pref"] else ""),
            attribute_name
        )
        for (kind, properties_list) in sorted(css_properties.items())
        for (property_name, data) in sorted(properties_list.items())
        for attribute_name in attribute_names(property_name)
    )
    parser.parse(idl, "CSSStyleDeclaration_generated.webidl")


def attribute_names(property_name):
    # https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-dashed-attribute
    if property_name != "float":
        yield property_name
    else:
        yield "_float"

    # https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-camel-cased-attribute
    if "-" in property_name:
        yield "".join(camel_case(property_name))

    # https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-webkit-cased-attribute
    if property_name.startswith("-webkit-"):
        yield "".join(camel_case(property_name), True)


# https://drafts.csswg.org/cssom/#css-property-to-idl-attribute
def camel_case(chars, webkit_prefixed=False):
    if webkit_prefixed:
        chars = chars[1:]
    next_is_uppercase = False
    for c in chars:
        if c == '-':
            next_is_uppercase = True
        elif next_is_uppercase:
            next_is_uppercase = False
            # Should be ASCII-uppercase, but all non-custom CSS property names are within ASCII
            yield c.upper()
        else:
            yield c


if __name__ == "__main__":
    main()
