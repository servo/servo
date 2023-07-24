from typing import Any, Mapping

from ._module import BidiModule, command


class Browser(BidiModule):
    @command
    def close(self) -> Mapping[str, Any]:
        return {}
