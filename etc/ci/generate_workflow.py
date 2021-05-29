# Copyright 2021 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os.path
import sys

BASE = os.path.dirname(__file__.replace('\\', '/'))
sys.path.insert(0, os.path.join(
    BASE, "..", "..", "components", "style", "properties", "Mako-1.1.2-py2.py3-none-any.whl"
))

from mako import exceptions  # noqa
from mako.lookup import TemplateLookup  # noqa
from mako.template import Template  # noqa


def abort(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def render(filename, **context):
    try:
        lookup = TemplateLookup(directories=[BASE],
                                input_encoding="utf8",
                                strict_undefined=True)
        template = Template(open(os.path.join(BASE, filename), "rb").read(),
                            filename=filename,
                            input_encoding="utf8",
                            lookup=lookup,
                            strict_undefined=True)
        # Uncomment to debug generated Python code:
        # write("/tmp", "mako_%s.py" % os.path.basename(filename), template.code)
        return template.render(**context)
    except Exception:
        # Uncomment to see a traceback in generated Python code:
        # raise
        abort(exceptions.text_error_template().render())


def main():
    with open(os.path.join(".github", "workflows", "main.yml"), 'w') as f:
        f.write(render(
            'workflow.mako',
            total_chunks=20,
            REPOSITORY_NAME="${{ github.event.repository.name }}",
            CODESIGN_CERT="${{ secrets.WINDOWS_CODESIGN_CERT }}",
        ))


if __name__ == "__main__":
    main()
