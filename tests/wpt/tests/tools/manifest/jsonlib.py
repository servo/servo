import re
import json
from typing import Any, AnyStr, Callable, Dict, IO, Text


__all__ = ["load", "dump_local", "dump_local", "dump_dist", "dumps_dist"]


try:
    import ujson
except ImportError:
    has_ujson = False
else:
    has_ujson = True

#
# load
#

if has_ujson:
    load: Callable[[IO[AnyStr]], Any] = ujson.load

else:
    load = json.load


#
# loads
#

if has_ujson:
    loads: Callable[[AnyStr], Any] = ujson.loads

else:
    loads = json.loads


#
# dump/dumps_local options for some libraries
#
_ujson_dump_local_kwargs: Dict[str, Any] = {
    'ensure_ascii': False,
    'escape_forward_slashes': False,
    'indent': 1,
    'reject_bytes': True,
}


_json_dump_local_kwargs: Dict[str, Any] = {
    'ensure_ascii': False,
    'indent': 1,
    'separators': (',', ': '),
}


#
# dump_local (for local, non-distributed usage of JSON)
#

if has_ujson:
    def dump_local(obj: Any, fp: IO[str]) -> None:
        return ujson.dump(obj, fp, **_ujson_dump_local_kwargs)

else:
    def dump_local(obj: Any, fp: IO[str]) -> None:
        return json.dump(obj, fp, **_json_dump_local_kwargs)


#
# dumps_local (for local, non-distributed usage of JSON)
#

if has_ujson:
    def dumps_local(obj: Any) -> Text:
        return ujson.dumps(obj, **_ujson_dump_local_kwargs)

else:
    def dumps_local(obj: Any) -> Text:
        return json.dumps(obj, **_json_dump_local_kwargs)


#
# dump/dumps_dist (for distributed usage of JSON where files should safely roundtrip)
#

_ujson_dump_dist_kwargs: Dict[str, Any] = {
    'sort_keys': True,
    'indent': 1,
    'reject_bytes': True,
    'escape_forward_slashes': False,
}


_json_dump_dist_kwargs: Dict[str, Any] = {
    'sort_keys': True,
    'indent': 1,
    'separators': (',', ': '),
}


if has_ujson:
    if ujson.dumps([], indent=1) == "[]":
        # optimistically see if https://github.com/ultrajson/ultrajson/issues/429 is fixed
        def _ujson_fixup(s: str) -> str:
            return s
    else:
        _ujson_fixup_re = re.compile(r"([\[{])[\n\x20]+([}\]])")

        def _ujson_fixup(s: str) -> str:
            return _ujson_fixup_re.sub(
                lambda m: m.group(1) + m.group(2),
                s
            )

    def dump_dist(obj: Any, fp: IO[str]) -> None:
        fp.write(_ujson_fixup(ujson.dumps(obj, **_ujson_dump_dist_kwargs)))

    def dumps_dist(obj: Any) -> Text:
        return _ujson_fixup(ujson.dumps(obj, **_ujson_dump_dist_kwargs))
else:
    def dump_dist(obj: Any, fp: IO[str]) -> None:
        json.dump(obj, fp, **_json_dump_dist_kwargs)

    def dumps_dist(obj: Any) -> Text:
        return json.dumps(obj, **_json_dump_dist_kwargs)
