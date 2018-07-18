"""
Invoke tasks to help with pytest development and release process.
"""

import invoke

from . import generate


ns = invoke.Collection(generate)
