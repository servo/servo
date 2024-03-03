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


class CookieFilter(Dict[str, Any]):
    def __init__(
        self,
        name: Optional[str] = None,
        value: Optional[NetworkBytesValue] = None,
        domain: Optional[str] = None,
        path: Optional[str] = None,
        http_only: Optional[bool] = None,
        secure: Optional[bool] = None,
        same_site: Optional[str] = None,
        size: Optional[int] = None,
        expiry: Optional[int] = None,
    ):
        if name is not None:
            self["name"] = name
        if value is not None:
            self["value"] = value
        if domain is not None:
            self["domain"] = domain
        if path is not None:
            self["path"] = path
        if http_only is not None:
            self["httpOnly"] = http_only
        if secure is not None:
            self["secure"] = secure
        if same_site is not None:
            self["sameSite"] = same_site
        if size is not None:
            self["size"] = size
        if expiry is not None:
            self["expiry"] = expiry


class Storage(BidiModule):
    @command
    def get_cookies(
        self,
        filter: Optional[CookieFilter] = None,
        partition: Optional[PartitionDescriptor] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if filter is not None:
            params["filter"] = filter
        if partition is not None:
            params["partition"] = partition
        return params

    @command
    def delete_cookies(
        self,
        filter: Optional[CookieFilter] = None,
        partition: Optional[PartitionDescriptor] = None,
    ) -> Mapping[str, Any]:
        params: MutableMapping[str, Any] = {}

        if filter is not None:
            params["filter"] = filter
        if partition is not None:
            params["partition"] = partition
        return params

    @command
    def set_cookie(
            self,
            cookie: PartialCookie,
            partition: Optional[PartitionDescriptor] = None
    ) -> Mapping[str, Any]:
        """
        Use with caution: this command will not clean the cookie up after the test is done, which can lead to unexpected
        test failures. Use `set_cookie` fixture instead.
        """
        params: MutableMapping[str, Any] = {
            "cookie": cookie
        }
        if partition is not None:
            params["partition"] = partition
        return params
