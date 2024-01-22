#!/usr/bin/env python3

import argparse
import json
import logging
import os

from dataclasses import dataclass
from pathlib import Path
from typing import Any, List, Optional

from ..manifest.item import SupportFile
from ..manifest.sourcefile import SourceFile
from ..metadata.yaml.load import load_data_to_dict
from ..web_features.web_feature_map import WebFeatureToTestsDirMapper, WebFeaturesMap
from .. import localpaths
from ..metadata.webfeatures.schema import WEB_FEATURES_YML_FILENAME, WebFeaturesFile

"""
This command generates a manifest file containing a mapping of web-feature
identifiers to test paths.

The web-feature identifiers are sourced from https://github.com/web-platform-dx/web-features.
They are used in WEB_FEATURES.yml files located throughout the WPT source code.
Each file defines which test files correspond to a specific identifier.
Refer to RFC 163 (https://github.com/web-platform-tests/rfcs/pull/163) for more
file details.

This command processes all WEB_FEATURES.yml files, extracts the list of test
paths from the test files, and writes them to a manifest file. The manifest
file maps web-feature identifiers to their corresponding test paths.

The file written is a JSON file. An example file looks like:

{
    "version": 1,
    "data": {
        "async-clipboard": [
            "/clipboard-apis/async-custom-formats-write-fail.tentative.https.html",
            "/clipboard-apis/async-custom-formats-write-read-web-prefix.tentative.https.html"
        ],
        "idle-detection": [
            "/idle-detection/basics.tentative.https.window.html",
            "/idle-detection/idle-detection-allowed-by-permissions-policy-attribute-redirect-on-load.https.sub.html"
        ]
    }
}


The JSON Schema for the file format can be found at MANIFEST_SCHEMA.json

This file does not follow the same format as the original manifest file,
MANIFEST.json.
"""

logger = logging.getLogger(__name__)

MANIFEST_FILE_NAME = "WEB_FEATURES_MANIFEST.json"


def abs_path(path: str) -> str:
    return os.path.abspath(os.path.expanduser(path))

def create_parser() -> argparse.ArgumentParser:
    """
    Creates an argument parser for the script.

    Returns:
        argparse.ArgumentParser: The configured argument parser.
    """
    parser = argparse.ArgumentParser(
        description="Maps tests to web-features within a repo root."
    )
    parser.add_argument(
        "-p", "--path", type=abs_path, help="Path to manifest file.")
    return parser


def find_all_test_files_in_dir(root_dir: str, rel_dir_path: str, url_base: str) -> List[SourceFile]:
    """
    Finds all test files within a given directory.

    Ignores any SourceFiles that are marked as non_test or the type
    is SupportFile.item_type

    Args:
        root_dir (str): The root directory of the repository.
        rel_dir_path (str): The relative path of the directory to search.
        url_base (str): Base url to use as the mount point for tests in this manifest.

    Returns:
        List[SourceFile]: A list of SourceFile objects representing the found test files.
    """
    rv: List[SourceFile] = []
    full_dir_path = os.path.join(root_dir, rel_dir_path)
    for file in os.listdir(full_dir_path):
        full_path = os.path.join(full_dir_path, file)
        rel_file_path = os.path.relpath(full_path, root_dir)
        source_file = SourceFile(root_dir, rel_file_path, url_base)
        if not source_file.name_is_non_test and source_file.type != SupportFile.item_type:
            rv.append(source_file)
    return rv

@dataclass
class CmdConfig():
    """
    Configuration for the command-line options.
    """

    repo_root: str  # The root directory of the WPT repository
    url_base: str  # Base URL used when converting file paths to urls


def map_tests_to_web_features(
        cmd_cfg: CmdConfig,
        rel_dir_path: str,
        result: WebFeaturesMap,
        prev_inherited_features: List[str] = []) -> None:
    """
    Recursively maps tests to web-features within a directory structure.

    Args:
        cmd_cfg (CmdConfig): The configuration for the command-line options.
        rel_dir_path (str): The relative path of the directory to process.
        result (WebFeaturesMap): The object to store the mapping results.
        prev_inherited_features (List[str], optional): A list of inherited web-features from parent directories. Defaults to [].
    """
    # Sometimes it will add a . at the beginning. Let's resolve the absolute path to disambiguate.
    # current_path = Path(os.path.join(cmd_cfg.repo_root, rel_dir_path)).resolve()
    current_dir = str(Path(os.path.join(cmd_cfg.repo_root, rel_dir_path)).resolve())

    # Create a copy that may be built upon or cleared during this iteration.
    inherited_features = prev_inherited_features.copy()

    rel_dir_path = os.path.relpath(current_dir, cmd_cfg.repo_root)

    web_feature_yml_full_path = os.path.join(current_dir, WEB_FEATURES_YML_FILENAME)
    web_feature_file: Optional[WebFeaturesFile] = None
    if os.path.isfile(web_feature_yml_full_path):
        try:
            web_feature_file = WebFeaturesFile(load_data_to_dict(
                open(web_feature_yml_full_path, "rb")))
        except Exception as e:
            raise e

    WebFeatureToTestsDirMapper(
        find_all_test_files_in_dir(cmd_cfg.repo_root, rel_dir_path, cmd_cfg.url_base),
        web_feature_file
    ).run(result, inherited_features)

    sub_dirs = [f for f in os.listdir(current_dir) if os.path.isdir(os.path.join(current_dir, f))]
    for sub_dir in sub_dirs:
        map_tests_to_web_features(
            cmd_cfg,
            os.path.join(rel_dir_path, sub_dir),
            result,
            inherited_features
        )

class WebFeatureManifestEncoder(json.JSONEncoder):
    """
    Custom JSON encoder.

    WebFeaturesMap contains a dictionary where the value is of type set.
    Sets cannot serialize to JSON by default. This encoder handles that by
    calling WebFeaturesMap's to_dict() method.
    """
    def default(self, obj: Any) -> Any:
        if isinstance(obj, WebFeaturesMap):
            return obj.to_dict()
        return super().default(obj)


def write_manifest_file(path: str, web_features_map: WebFeaturesMap) -> None:
    """
    Serializes the WebFeaturesMap to a JSON manifest file at the specified path.

    The generated JSON file adheres to the schema defined in the "MANIFEST_SCHEMA.json" file. The
    serialization process uses the custom `WebFeatureManifestEncoder` to ensure correct formatting.

    Args:
        path (str): The file path where the manifest file will be created or overwritten.
        web_features_map (WebFeaturesMap): The object containing the mapping between
                                           web-features and their corresponding test paths.
    """
    with open(path, "w") as outfile:
        outfile.write(
            json.dumps(
                {
                    "version": 1,
                    "data": web_features_map
                }, cls=WebFeatureManifestEncoder))


def main(venv: Any = None, **kwargs: Any) -> int:

    assert logger is not None

    repo_root = localpaths.repo_root
    url_base = "/"
    path = kwargs.get("path") or os.path.join(repo_root, MANIFEST_FILE_NAME)

    cmd_cfg = CmdConfig(repo_root, url_base)
    feature_map = WebFeaturesMap()
    map_tests_to_web_features(cmd_cfg, "", feature_map)
    write_manifest_file(path, feature_map)

    return 0
