#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os.path
import sys
import json

style = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(style, "Mako-0.9.1.zip"))
from mako.template import Template

template = Template(filename=os.path.join(style, "properties.mako.rs"), input_encoding='utf8')
template.render(PRODUCT='servo')
properties = dict(
    (p.name, {
        "flag": p.experimental,
        "shorthand": hasattr(p, "sub_properties")
    })
    for p in template.module.LONGHANDS + template.module.SHORTHANDS
)

json_dump = json.dumps(properties, indent=4)

#
# Resolve path to doc directory and write CSS properties and JSON.
#
servo_doc_path = os.path.abspath(os.path.join(style, '../', '../', 'target', 'doc', 'servo'))

# Ensure ./target/doc/servo exists
if not os.path.exists(servo_doc_path):
    os.makedirs(servo_doc_path)

with open(os.path.join(servo_doc_path, 'css-properties.json'), "w") as out_file:
    out_file.write(json_dump)

html_template = Template(filename=os.path.join(style, "properties.html.mako"), input_encoding='utf8')
with open(os.path.join(servo_doc_path, 'css-properties.html'), "w") as out_file:
    out_file.write(html_template.render(properties=properties))
