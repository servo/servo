from __future__ import annotations

import asyncio
import sys
from typing import Any, Dict


def loop_if_py_lt_38(loop: asyncio.AbstractEventLoop) -> Dict[str, Any]:
    """
    Helper for the removal of the loop argument in Python 3.10.

    """
    return {"loop": loop} if sys.version_info[:2] < (3, 8) else {}
