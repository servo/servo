# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import sys

from mako import exceptions
from mako.lookup import TemplateLookup
from mako.template import Template

try:
    lookup = TemplateLookup(directories=['.'])
    template = Template(open(os.environ['TEMPLATE'], 'rb').read(),
                        input_encoding='utf8',
                        lookup=lookup)
    print(template.render(PRODUCT=os.environ['PRODUCT']).encode('utf8'))
except:
    sys.stderr.write(exceptions.text_error_template().render().encode('utf8'))
    sys.exit(1)
