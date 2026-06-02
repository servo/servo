from __future__ import annotations

import sys
import typing

from .imports import lazy_import
from .version import version as websockets_version


# For backwards compatibility:


# When type checking, import non-deprecated aliases eagerly. Else, import on demand.
if typing.TYPE_CHECKING:
    from .datastructures import Headers, MultipleValuesError  # noqa: F401
else:
    lazy_import(
        globals(),
        # Headers and MultipleValuesError used to be defined in this module.
        aliases={
            "Headers": ".datastructures",
            "MultipleValuesError": ".datastructures",
        },
        deprecated_aliases={
            "read_request": ".legacy.http",
            "read_response": ".legacy.http",
        },
    )


__all__ = ["USER_AGENT"]


PYTHON_VERSION = "{}.{}".format(*sys.version_info)
USER_AGENT = f"Python/{PYTHON_VERSION} websockets/{websockets_version}"
