from typing import Any, Optional, Mapping, MutableMapping, Union
from webdriver.bidi.undefined import UNDEFINED, Undefined

from ._module import BidiModule, command


class Permissions(BidiModule):
    @command
    def set_permission(self,
                    descriptor: Union[Optional[Mapping[str, Any]], Undefined] = UNDEFINED,
                    state: Union[Optional[str], Undefined] = UNDEFINED,
                    origin: Union[Optional[str], Undefined] = UNDEFINED,
                    user_context: Union[Optional[str], Undefined] = UNDEFINED) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "descriptor": descriptor,
            "state": state,
            "origin": origin,
            "userContext": user_context,
        }
        return params
