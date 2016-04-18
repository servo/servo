# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import os.path
import re
import sys

BASE = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(BASE, "Mako-0.9.1.zip"))

from mako import exceptions
from mako.template import Template


def main():
    usage = "Usage: %s [ servo | gecko ] [ rust | html ]" % sys.argv[0]
    if len(sys.argv) < 3:
        abort(usage)
    product = sys.argv[1]
    output = sys.argv[2]
    if product not in ["servo", "gecko"] or output not in ["rust", "html"]:
        abort(usage)

    template, rust = render("properties.mako.rs", PRODUCT=product)
    if output == "rust":
        write(os.environ["OUT_DIR"], "properties.rs", rust)
    elif output == "html":
        write_html(template)


def abort(message):
    sys.stderr.write(message + b"\n")
    sys.exit(1)


def render(name, **context):
    try:
        template = Template(open(os.path.join(BASE, name), "rb").read(), input_encoding="utf8")
        return template, template.render(**context).encode("utf8")
    except:
        abort(exceptions.text_error_template().render().encode("utf8"))


def write(directory, filename, content):
    if not os.path.exists(directory):
        os.makedirs(directory)
    open(os.path.join(directory, filename), "wb").write(content)


def write_html(template):
    properties = dict(
        (p.name, {
            "flag": p.experimental,
            "shorthand": hasattr(p, "sub_properties")
        })
        for p in template.module.LONGHANDS + template.module.SHORTHANDS
    )
    _, html = render("properties.html.mako", properties=properties)

    doc_servo = os.path.join(BASE, "..", "..", "target", "doc", "servo")
    write(doc_servo, "css-properties.json", json.dumps(properties, indent=4))
    write(doc_servo, "css-properties.html", html)


if __name__ == "__main__":
    main()
