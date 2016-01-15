#!/bin/env python2.7

import os.path
import sys

style = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(style, "Mako-0.9.1.zip"))

from mako.template import Template
template = Template(filename=os.path.join(style, "properties.mako.rs"), input_encoding='utf8')
template.render()
properties = template.module.LONGHANDS + template.module.SHORTHANDS
for name in sorted(p.name for p in properties):
    print(name)
