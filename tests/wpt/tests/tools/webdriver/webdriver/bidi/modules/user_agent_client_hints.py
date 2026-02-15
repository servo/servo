from typing import Any, Dict, List, Mapping

from ._module import BidiModule, command
from ..undefined import UNDEFINED, Maybe, Nullable


class UserAgentClientHints(BidiModule):
    @command
    def set_client_hints_override(
            self,
            client_hints: Nullable[Dict[str, Any]],
            contexts: Maybe[List[str]] = UNDEFINED,
            user_contexts: Maybe[List[str]] = UNDEFINED,
    ) -> Mapping[str, Any]:
        return {
            "clientHints": client_hints,
            "contexts": contexts,
            "userContexts": user_contexts,
        }
