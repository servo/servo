import json
import os
import sys

from mako import exceptions
from mako.lookup import TemplateLookup
from mako.template import Template

try:
    style_template = Template(filename=os.environ['STYLE_TEMPLATE'],
                              input_encoding='utf8')
    style_template.render(PRODUCT='gecko')

    geckolib_template = Template(filename=os.environ['GECKOLIB_TEMPLATE'], input_encoding='utf8')
    output = geckolib_template.render(STYLE_STRUCTS = style_template.module.STYLE_STRUCTS,
                                      to_rust_ident = style_template.module.to_rust_ident)
    print(output.encode('utf8'))
except:
    sys.stderr.write(exceptions.text_error_template().render().encode('utf8'))
    sys.exit(1)
