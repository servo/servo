# mypy: allow-untyped-defs

from typing import Dict, Iterable, List, Tuple

from wptserve.utils import isomorphic_decode


class H3Headers(Dict[str, str]):
    """Header store that adapts HTTP/3 headers to pywebsocket3 expectations."""

    _H3_PSEUDO_HEADERS = {
        'method', 'scheme', 'host', 'path', 'authority', 'status', 'protocol'
    }

    def __init__(self, headers: Iterable[Tuple[bytes, bytes]]) -> None:
        super().__init__()
        self.raw_headers: Dict[str, str] = {}
        for raw_key, raw_value in headers:
            key = isomorphic_decode(raw_key)
            value = isomorphic_decode(raw_value)
            if key == "cookie" and key in self.raw_headers:
                value = self.raw_headers[key] + "; " + value
            self.raw_headers[key] = value
            dict.__setitem__(self, key, value)

            dict.__setitem__(self, self._normalize_key(key), value)

    @staticmethod
    def _normalize_key(key: str) -> str:
        if key.startswith(':') and key[1:] in H3Headers._H3_PSEUDO_HEADERS:
            return key[1:]
        return key

    # TODO This does not seem relevant for H3 headers, so using a dummy function for now
    def getallmatchingheaders(self, _header: str) -> List[str]:
        return ['dummy function']
