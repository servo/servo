# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import os.path
import sys

BASE = os.path.dirname(__file__.replace('\\', '/'))
sys.path.insert(0, os.path.join(BASE, "Mako-0.9.1.zip"))
sys.path.insert(0, BASE)  # For importing `data.py`

from mako import exceptions
from mako.lookup import TemplateLookup
from mako.template import Template

import data


def main():
    usage = "Usage: %s [ servo | gecko ] [ style-crate | html ] [ testing | regular ]" % sys.argv[0]
    if len(sys.argv) < 4:
        abort(usage)
    product = sys.argv[1]
    output = sys.argv[2]
    testing = sys.argv[3] == "testing"

    if product not in ["servo", "gecko"] or output not in ["style-crate", "geckolib", "html"]:
        abort(usage)

    properties = data.PropertiesData(product=product, testing=testing)
    template = os.path.join(BASE, "properties.mako.rs")
    rust = render(template, product=product, data=properties, __file__=template)
    if output == "style-crate":
        write(os.environ["OUT_DIR"], "properties.rs", rust)
        write(os.environ["OUT_DIR"], "static_ids.txt", static_ids(properties))
        if product == "gecko":
            template = os.path.join(BASE, "gecko.mako.rs")
            rust = render(template, data=properties)
            write(os.environ["OUT_DIR"], "gecko_properties.rs", rust)
    elif output == "html":
        write_html(properties)


def abort(message):
    sys.stderr.write(message + b"\n")
    sys.exit(1)


def render(filename, **context):
    try:
        lookup = TemplateLookup(directories=[BASE])
        template = Template(open(filename, "rb").read(),
                            filename=filename,
                            input_encoding="utf8",
                            lookup=lookup,
                            strict_undefined=True)
        # Uncomment to debug generated Python code:
        # write("/tmp", "mako_%s.py" % os.path.basename(filename), template.code)
        return template.render(**context).encode("utf8")
    except:
        # Uncomment to see a traceback in generated Python code:
        # raise
        abort(exceptions.text_error_template().render().encode("utf8"))


def write(directory, filename, content):
    if not os.path.exists(directory):
        os.makedirs(directory)
    open(os.path.join(directory, filename), "wb").write(content)


def static_id_generator(properties):
    for kind, props in [("Longhand", properties.longhands),
                        ("Shorthand", properties.shorthands)]:
        for p in props:
            yield "%s\tStaticId::%s(%sId::%s)" % (p.name, kind, kind, p.camel_case)
            for alias in p.alias:
                yield "%s\tStaticId::%s(%sId::%s)" % (alias, kind, kind, p.camel_case)


def static_ids(properties):
    return '\n'.join(static_id_generator(properties))


def write_html(properties):
    properties = dict(
        (p.name, {
            "flag": p.experimental,
            "shorthand": hasattr(p, "sub_properties")
        })
        for p in properties.longhands + properties.shorthands
    )
    doc_servo = os.path.join(BASE, "..", "..", "..", "target", "doc", "servo")
    html = render(os.path.join(BASE, "properties.html.mako"), properties=properties)
    write(doc_servo, "css-properties.html", html)
    write(doc_servo, "css-properties.json", json.dumps(properties, indent=4))


if __name__ == "__main__":
    main()
