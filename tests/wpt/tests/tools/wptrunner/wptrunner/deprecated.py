from __future__ import annotations

import sys
from typing import TYPE_CHECKING, TypeVar

if sys.version_info >= (3, 13):
    from warnings import deprecated as deprecated
elif TYPE_CHECKING:
    from typing_extensions import deprecated as deprecated
else:
    _T = TypeVar("_T")

    class deprecated:
        def __init__(
            self,
            message: str,
            /,
            *,
            category: type[Warning] | None = DeprecationWarning,
            stacklevel: int = 1,
        ) -> None:
            self.message = message
            self.category = category
            self.stacklevel = stacklevel

        def __call__(self, f: _T) -> _T:
            return f
