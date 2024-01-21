from typing import Any, Dict, Mapping, MutableMapping, Optional, Union
from ._module import BidiModule, command
from webdriver.bidi.modules.network import NetworkBytesValue


class BrowsingContextPartitionDescriptor(Dict[str, Any]):
    def __init__(self, context: str):
        dict.__init__(self, type="context", context=context)


class StorageKeyPartitionDescriptor(Dict[str, Any]):
    def __init__(self, user_context: Optional[str] = None,
                 source_origin: Optional[str] = None):
        dict.__init__(self, type="storageKey")
        if user_context is not None:
            self["userContext"] = user_context
        if source_origin is not None:
            self["sourceOrigin"] = source_origin


class PartialCookie(Dict[str, Any]):
    def __init__(
            self,
            name: str,
            value: NetworkBytesValue,
            domain: str,
            path: Optional[str] = None,
            http_only: Optional[bool] = None,
            secure: Optional[bool] = None,
            same_site: Optional[str] = None,
            expiry: Optional[int] = None,
    ):
        dict.__init__(self, name=name, value=value, domain=domain)
        if path is not None:
            self["path"] = path
        if http_only is not None:
            self["httpOnly"] = http_only
        if secure is not None:
            self["secure"] = secure
        if same_site is not None:
            self["sameSite"] = same_site
        if expiry is not None:
            self["expiry"] = expiry


PartitionDescriptor = Union[StorageKeyPartitionDescriptor, BrowsingContextPartitionDescriptor]


class Storage(BidiModule):

    # TODO: extend with `filter`.
    @command
    def get_cookies(self, partition: Optional[PartitionDescriptor] = None) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}
        if partition is not None:
            params["partition"] = partition
        return params

    @command
    def set_cookie(
            self,
            cookie: PartialCookie,
            partition: Optional[PartitionDescriptor] = None
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {
            "cookie": cookie
        }
        if partition is not None:
            params["partition"] = partition
        return params
