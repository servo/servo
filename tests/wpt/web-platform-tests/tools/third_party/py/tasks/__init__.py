"""
Invoke tasks to help with pytest development and release process.
"""

import invoke

from . import vendoring


ns = invoke.Collection(
    vendoring
)
