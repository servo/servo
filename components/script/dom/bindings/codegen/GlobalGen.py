# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# We do one global pass over all the WebIDL to generate our prototype enum
# and generate information for subsequent phases.

import sys
import os
import json
sys.path.append(os.path.join(".", "parser"))
sys.path.append(os.path.join(".", "ply"))
import WebIDL
import cPickle
from Configuration import Configuration
from CodegenRust import GlobalGenRoots, replaceFileIfChanged


def generate_file(config, name, filename):
    root = getattr(GlobalGenRoots, name)(config)
    code = root.define()

    if replaceFileIfChanged(filename, code):
        print "Generating %s" % (filename)
    else:
        print "%s hasn't changed - not touching it" % (filename)


def main():
    # Parse arguments.
    from optparse import OptionParser
    usageString = "usage: %prog [options] configFile outputdir webidldir cssProperties.json docServoDir [files]"
    o = OptionParser(usage=usageString)
    o.add_option("--cachedir", dest='cachedir', default=None,
                 help="Directory in which to cache lex/parse tables.")
    o.add_option("--filelist", dest='filelist', default=None,
                 help="A file containing the list (one per line) of webidl files to process.")
    (options, args) = o.parse_args()

    if len(args) < 2:
        o.error(usageString)

    configFile = args[0]
    outputdir = args[1]
    baseDir = args[2]
    css_properties_json = args[3]
    doc_servo = args[4]
    if options.filelist is not None:
        fileList = [l.strip() for l in open(options.filelist).xreadlines()]
    else:
        fileList = args[3:]

    # Parse the WebIDL.
    parser = WebIDL.Parser(options.cachedir)
    for filename in fileList:
        fullPath = os.path.normpath(os.path.join(baseDir, filename))
        with open(fullPath, 'rb') as f:
            lines = f.readlines()
        parser.parse(''.join(lines), fullPath)

    add_css_properties_attributes(fileList, css_properties_json, parser)

    parserResults = parser.finish()

    # Write the parser results out to a pickle.
    resultsPath = os.path.join(outputdir, 'ParserResults.pkl')
    with open(resultsPath, 'wb') as resultsFile:
        cPickle.dump(parserResults, resultsFile, -1)

    # Load the configuration.
    config = Configuration(configFile, parserResults)

    to_generate = [
        ('PrototypeList', 'PrototypeList.rs'),
        ('RegisterBindings', 'RegisterBindings.rs'),
        ('InterfaceObjectMap', 'InterfaceObjectMap.rs'),
        ('InterfaceObjectMapData', 'InterfaceObjectMapData.json'),
        ('InterfaceTypes', 'InterfaceTypes.rs'),
        ('InheritTypes', 'InheritTypes.rs'),
        ('Bindings', os.path.join('Bindings', 'mod.rs')),
        ('UnionTypes', 'UnionTypes.rs'),
    ]

    for name, filename in to_generate:
        generate_file(config, name, os.path.join(outputdir, filename))

    generate_file(config, 'SupportedDomApis', os.path.join(doc_servo, 'apis.html'))


def add_css_properties_attributes(webidl_files, css_properties_json, parser):
    for filename in webidl_files:
        if os.path.basename(filename) == "CSSStyleDeclaration.webidl":
            break
    else:
        return

    css_properties = json.load(open(css_properties_json, "rb"))
    idl = "partial interface CSSStyleDeclaration {\n%s\n};\n" % "\n".join(
        "  [%sCEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString %s;" % (
            ('Pref="%s", ' % data["pref"] if data["pref"] else ""),
            attribute_name
        )
        for (kind, properties_list) in sorted(css_properties.items())
        for (property_name, data) in sorted(properties_list.items())
        for attribute_name in attribute_names(property_name)
    )
    parser.parse(idl.encode("utf-8"), "CSSStyleDeclaration_generated.webidl")


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


if __name__ == '__main__':
    main()
