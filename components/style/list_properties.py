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
template.render()
properties = dict(
    (p.name, {
        "flag": p.experimental,
        "shorthand": hasattr(p, "sub_properties")
    })
    for p in template.module.LONGHANDS + template.module.SHORTHANDS
)
print(json.dumps(properties, indent=4))
