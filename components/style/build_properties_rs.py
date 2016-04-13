import os
import sys

from mako.template import Template
from mako import exceptions

try:
    template = Template(open(os.environ['TEMPLATE'], 'rb').read(), input_encoding='utf8')
    print(template.render(PRODUCT=os.environ['PRODUCT']).encode('utf8'))
except:
    sys.stderr.write(exceptions.text_error_template().render().encode('utf8'))
    sys.exit(1)
