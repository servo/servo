import argparse
import json
import os
from collections import defaultdict

from .manifest import load_and_update, Manifest
from .log import get_logger

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Dict
    from typing import Iterable
    from typing import List
    from typing import Text

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))

logger = get_logger()


def abs_path(path):
    # type: (str) -> str
    return os.path.abspath(os.path.expanduser(path))


def create_parser():
    # type: () -> argparse.ArgumentParser
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-p", "--path", type=abs_path, help="Path to manifest file.")
    parser.add_argument(
        "--src-root", type=abs_path, default=None, help="Path to root of sourcetree.")
    parser.add_argument(
        "--tests-root", type=abs_path, default=wpt_root, help="Path to root of tests.")
    parser.add_argument(
        "--no-update", dest="update", action="store_false", default=True,
        help="Don't update manifest before continuing")
    parser.add_argument(
        "-r", "--rebuild", action="store_true", default=False,
        help="Force a full rebuild of the manifest.")
    parser.add_argument(
        "--url-base", action="store", default="/",
        help="Base url to use as the mount point for tests in this manifest.")
    parser.add_argument(
        "--cache-root", action="store", default=os.path.join(wpt_root, ".wptcache"),
        help="Path in which to store any caches (default <tests_root>/.wptcache/)")
    parser.add_argument(
        "--json", action="store_true", default=False,
        help="Output as JSON")
    parser.add_argument(
        "test_ids", action="store", nargs="+",
        help="Test ids for which to get paths")
    return parser


def get_path_id_map(src_root, tests_root, manifest_file, test_ids):
    # type: (Text, Text, Manifest, Iterable[Text]) -> Dict[Text, List[Text]]
    test_ids = set(test_ids)
    path_id_map = defaultdict(list)  # type: Dict[Text, List[Text]]

    compute_rel_path = src_root != tests_root

    for item_type, path, tests in manifest_file:
        for test in tests:
            if test.id in test_ids:
                if compute_rel_path:
                    rel_path = os.path.relpath(os.path.join(tests_root, path),
                                               src_root)
                else:
                    rel_path = path
                path_id_map[rel_path].append(test.id)
    return path_id_map


def get_paths(**kwargs):
    # type: (**Any) -> Dict[Text, List[Text]]
    tests_root = kwargs["tests_root"]
    assert tests_root is not None
    path = kwargs["path"]
    if path is None:
        path = os.path.join(kwargs["tests_root"], "MANIFEST.json")
    src_root = kwargs["src_root"]
    if src_root is None:
        src_root = tests_root

    manifest_file = load_and_update(tests_root,
                                    path,
                                    kwargs["url_base"],
                                    update=kwargs["update"],
                                    rebuild=kwargs["rebuild"],
                                    cache_root=kwargs["cache_root"])

    return get_path_id_map(src_root, tests_root, manifest_file, kwargs["test_ids"])


def write_output(path_id_map, as_json):
    # type: (Dict[Text, List[Text]], bool) -> None
    if as_json:
        print(json.dumps(path_id_map))
    else:
        for path, test_ids in sorted(path_id_map.items()):
            print(path)
            for test_id in sorted(test_ids):
                print("  " + test_id)


def run(**kwargs):
    # type: (**Any) -> None
    path_id_map = get_paths(**kwargs)
    write_output(path_id_map, as_json=kwargs["json"])
