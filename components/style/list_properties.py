#!/bin/env python2.7

import os.path
import sys
import json
import operator

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
