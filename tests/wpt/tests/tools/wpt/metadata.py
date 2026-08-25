import argparse
import logging
import os
import re
from typing import Any, Dict, List, Optional, Mapping, Sequence, Set, Union

import pydantic
import yaml
from pydantic import BaseModel


from ..manifest import manifest
from .virtualenv import Virtualenv

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

class UrlResultsModel(BaseModel):
    model_config = pydantic.ConfigDict(extra='forbid')

    test: str
    subtest: Optional[str] = None
    status: Optional[str] = None


class UrlLinkModel(BaseModel):
    model_config = pydantic.ConfigDict(extra='forbid')

    url: str
    results: List[UrlResultsModel]
    product: Optional[str] = None


class LabelResultsModel(BaseModel):
    test: str


class LabelLinkModel(BaseModel):
    label: str
    results: List[LabelResultsModel]


class MetadataModel(BaseModel):
    links: List[Union[LabelLinkModel, UrlLinkModel]]


def load_metadata_map(src_dir: str) -> Mapping[str, MetadataModel]:
    rv = {}
    for dir_path, dir_names, file_names in os.walk(src_dir):
        if "META.yml" not in file_names:
            continue

        id_prefix = os.path.relpath(dir_path, src_dir).replace(os.path.sep, "/") + "/"
        if id_prefix[0] != "/":
            id_prefix = "/" + id_prefix
        meta_path = os.path.join(dir_path, "META.yml")

        with open(meta_path) as f:
            data = yaml.safe_load(f)
        try:
            rv[id_prefix] = MetadataModel.model_validate(data)
        except Exception as e:
            logging.critical(f"Error validating metadata {meta_path}")
            raise e
    return rv


def get_all_tests(metadata_map: Mapping[str, MetadataModel]) -> Set[str]:
    rv = set()

    for id_prefix, metadata in metadata_map.items():
        for link in metadata.links:
            for result in link.results:
                if result.test != "*":
                    test_id = id_prefix + result.test
                    rv.add(test_id)
    return rv


def get_labelled_tests(
        metadata_map: Mapping[str, MetadataModel],
        label_patterns: Sequence[Union[str, re.Pattern[Any]]]
) -> Mapping[str, Set[str]]:
    rv: Dict[str, Set[str]] = {}
    for id_prefix, metadata in metadata_map.items():
        for link in metadata.links:
            if isinstance(link, LabelLinkModel):
                for label in label_patterns:
                    if (isinstance(label, str) and link.label == label or
                        isinstance(label, re.Pattern) and label.match(link.label) is not None):
                        if link.label not in rv:
                            rv[link.label] = set()

                        for result in link.results:
                            rv[link.label].add(id_prefix + result.test)
                        break
    return rv


def get_parser_validate() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", dest="manifest_path", help="Path to MANIFEST.json")
    parser.add_argument("metadata_path", help="Path to wpt metadata repository")
    return parser


def run_validate(venv: Virtualenv, metadata_path: str, manifest_path: Optional[str] = None) -> int:
    if manifest_path is None:
        manifest_path = os.path.join(wpt_root, "MANIFEST.json")
    wpt_manifest = manifest.load_and_update(wpt_root, manifest_path, "/")
    metadata_map = load_metadata_map(metadata_path)

    metadata_tests = get_all_tests(metadata_map)
    for _type, _rel_path, tests in wpt_manifest:
        for test in tests:
            metadata_tests.discard(test.id)

    if metadata_tests:
        tests_str = "\n".join(metadata_map)
        logging.error(f"The following tests were in metadata but not in the manifest:\n{tests_str}")
        return 1

    return 0


def get_parser_list() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", dest="manifest_path", help="Path to MANIFEST.json")
    parser.add_argument("metadata_path", help="Path to wpt metadata repository")
    parser.add_argument("labels", nargs="+", help="Regexp representing test labels to list")
    return parser


def run_list(venv: Virtualenv, metadata_path: str, labels: List[str], manifest_path: Optional[str] = None) -> int:
    if manifest_path is None:
        manifest_path = os.path.join(wpt_root, "MANIFEST.json")
    wpt_manifest = manifest.load_and_update(wpt_root, manifest_path, "/")

    metadata_map = load_metadata_map(metadata_path)
    label_patterns = [re.compile(item) for item in labels]
    tests_by_label = get_labelled_tests(metadata_map, label_patterns)
    all_labelled_tests = set()
    tests_by_id = {}
    for labelled_tests in tests_by_label.values():
        all_labelled_tests |= set(labelled_tests)

    for test_type, _rel_path, tests in wpt_manifest:
        for test in tests:
            if test.id in all_labelled_tests:
                tests_by_id[test.id] = (test_type, test)


    for label in sorted(tests_by_label):
        labelled_tests = tests_by_label[label]
        print(f"{label}\t{len(labelled_tests)}")
        for test in sorted(tests, key=lambda x:x.id):
            print(f"  {test}\t{tests_by_id[test.id][0]}")
    return 0
