# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import os.path
import re
import sys

BASE = os.path.dirname(__file__.replace('\\', '/'))
sys.path.insert(0, os.path.join(BASE, "Mako-0.9.1.zip"))
sys.path.insert(0, BASE)  # For importing `data.py`

from mako import exceptions
from mako.lookup import TemplateLookup
from mako.template import Template

import data

RE_PYTHON_ADDR = re.compile(r'<.+? object at 0x[0-9a-fA-F]+>')

OUT_DIR = os.environ.get("OUT_DIR", "")

STYLE_STRUCT_LIST = [
    "background",
    "border",
    "box",
    "color",
    "column",
    "counters",
    "effects",
    "font",
    "inherited_box",
    "inherited_table",
    "inherited_text",
    "inherited_ui",
    "inherited_svg",
    "list",
    "margin",
    "outline",
    "padding",
    "position",
    "table",
    "text",
    "ui",
    "svg",
    "xul",
]


def main():
    usage = ("Usage: %s [ servo | gecko ] [ style-crate | geckolib <template> | html ]" %
             sys.argv[0])
    if len(sys.argv) < 3:
        abort(usage)
    product = sys.argv[1]
    output = sys.argv[2]

    if product not in ["servo", "gecko"] or output not in ["style-crate", "geckolib", "html"]:
        abort(usage)

    properties = data.PropertiesData(product=product)
    files = {}
    for kind in ["longhands", "shorthands"]:
        files[kind] = {}
        for struct in STYLE_STRUCT_LIST:
            file_name = os.path.join(BASE, kind, "{}.mako.rs".format(struct))
            if kind == "shorthands" and not os.path.exists(file_name):
                files[kind][struct] = ""
                continue
            files[kind][struct] = render(
                file_name,
                product=product,
                data=properties,
            )
    properties_template = os.path.join(BASE, "properties.mako.rs")
    files["properties"] = render(
        properties_template,
        product=product,
        data=properties,
        __file__=properties_template,
        OUT_DIR=OUT_DIR,
    )
    if output == "style-crate":
        write(OUT_DIR, "properties.rs", files["properties"])
        for kind in ["longhands", "shorthands"]:
            for struct in files[kind]:
                write(
                    os.path.join(OUT_DIR, kind),
                    "{}.rs".format(struct),
                    files[kind][struct],
                )

        if product == "gecko":
            template = os.path.join(BASE, "gecko.mako.rs")
            rust = render(template, data=properties)
            write(OUT_DIR, "gecko_properties.rs", rust)
    elif output == "geckolib":
        if len(sys.argv) < 4:
            abort(usage)
        template = sys.argv[3]
        header = render(template, data=properties)
        sys.stdout.write(header)
    elif output == "html":
        write_html(properties)


def abort(message):
    sys.stderr.write(message + b"\n")
    sys.exit(1)


def render(filename, **context):
    try:
        lookup = TemplateLookup(directories=[BASE],
                                input_encoding="utf8",
                                strict_undefined=True)
        template = Template(open(filename, "rb").read(),
                            filename=filename,
                            input_encoding="utf8",
                            lookup=lookup,
                            strict_undefined=True)
        # Uncomment to debug generated Python code:
        # write("/tmp", "mako_%s.py" % os.path.basename(filename), template.code)
        return template.render(**context).encode("utf8")
    except Exception:
        # Uncomment to see a traceback in generated Python code:
        # raise
        abort(exceptions.text_error_template().render().encode("utf8"))


def write(directory, filename, content):
    if not os.path.exists(directory):
        os.makedirs(directory)
    full_path = os.path.join(directory, filename)
    open(full_path, "wb").write(content)

    python_addr = RE_PYTHON_ADDR.search(content)
    if python_addr:
        abort("Found \"{}\" in {} ({})".format(python_addr.group(0), filename, full_path))


def write_html(properties):
    properties = dict(
        (p.name, {
            "flag": p.servo_pref,
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
