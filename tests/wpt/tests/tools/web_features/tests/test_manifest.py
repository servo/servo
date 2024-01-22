# mypy: ignore-errors

import json
import os
from jsonschema import validate
from unittest.mock import ANY, Mock, call, mock_open, patch

import pytest

from ..manifest import create_parser, find_all_test_files_in_dir, main, map_tests_to_web_features, write_manifest_file, CmdConfig
from ..web_feature_map import WebFeatureToTestsDirMapper, WebFeaturesMap
from ...metadata.webfeatures.schema import WEB_FEATURES_YML_FILENAME
from ...manifest.sourcefile import SourceFile
from ...manifest.item import SupportFile, URLManifestItem
from ... import localpaths


@patch("os.listdir")
@patch("tools.web_features.manifest.SourceFile")
def test_find_all_test_files_in_dir(mock_source_file_class, mock_listdir):
    mock_listdir.return_value = ["test1.html", "support.py", "test2.html", "test3.html"]

    def create_source_file_mock(root_dir, rel_file_path, separator):
        source_file = Mock(spec=SourceFile)
        if rel_file_path.endswith("support.py"):
            source_file.name_is_non_test = True
            source_file.type = SupportFile.item_type
        else:
            source_file.name_is_non_test = False
        return source_file

    mock_source_file_class.side_effect = create_source_file_mock

    test_files = find_all_test_files_in_dir("root_dir", "rel_dir_path", "/")

    # Assert calls to the mocked constructor with expected arguments
    mock_source_file_class.assert_has_calls([
        call("root_dir", os.path.join("rel_dir_path", "test1.html"), "/"),
        call("root_dir", os.path.join("rel_dir_path", "support.py"), "/"),
        call("root_dir", os.path.join("rel_dir_path", "test2.html"), "/"),
        call("root_dir", os.path.join("rel_dir_path", "test3.html"), "/"),
    ])
    assert mock_source_file_class.call_count == 4


    # Assert attributes of the resulting test files
    assert all(
        not file.name_is_non_test and file.type != SupportFile.item_type
        for file in test_files
    )

    # Should only have 3 items instead of the original 4
    assert len(test_files) == 3

@patch("builtins.open", new_callable=mock_open, read_data="data")
@patch("os.listdir")
@patch("os.path.isdir")
@patch("os.path.isfile")
@patch("tools.web_features.manifest.load_data_to_dict", return_value={})
@patch("tools.web_features.manifest.find_all_test_files_in_dir")
@patch("tools.web_features.manifest.WebFeaturesFile")
@patch("tools.web_features.manifest.WebFeatureToTestsDirMapper", spec=WebFeatureToTestsDirMapper)
def test_map_tests_to_web_features_recursive(
    mock_mapper,
    mock_web_features_file,
    mock_find_all_test_files_in_dir,
    mock_load_data_to_dict,
    mock_isfile,
    mock_isdir,
    mock_listdir,
    mock_file
):
    def fake_listdir(path):
        if path.endswith("repo_root"):
            return ["subdir1", "subdir2"]
        elif path.endswith(os.path.join("repo_root", "subdir1")):
            return ["subdir1_1", "subdir1_2", WEB_FEATURES_YML_FILENAME]
        elif path.endswith(os.path.join("repo_root", "subdir1", "subdir1_1")):
            return [WEB_FEATURES_YML_FILENAME]
        elif path.endswith(os.path.join("repo_root", "subdir1", "subdir1_2")):
            return []
        elif path.endswith(os.path.join("repo_root", "subdir2")):
            return [WEB_FEATURES_YML_FILENAME]
        else:
            []
    mock_listdir.side_effect = fake_listdir

    def fake_isdir(path):
        if (path.endswith(os.path.join("repo_root", "subdir1")) or
        path.endswith(os.path.join("repo_root", "subdir1", "subdir1_1")) or
        path.endswith(os.path.join("repo_root", "subdir1", "subdir1_2")) or
        path.endswith(os.path.join("repo_root", "subdir2"))):
            return True
        return False
    mock_isdir.side_effect = fake_isdir

    def fake_isfile(path):
        if (path.endswith(os.path.join("repo_root", "subdir1", "WEB_FEATURES.yml")) or
        path.endswith(os.path.join("repo_root", "subdir1", "subdir1_1", "WEB_FEATURES.yml")) or
        path.endswith(os.path.join("repo_root", "subdir2", "WEB_FEATURES.yml"))):
            return True
        return False
    mock_isfile.side_effect = fake_isfile


    expected_root_files = [
        Mock(name="root_test_1"),
    ]

    expected_subdir1_files = [
        Mock(name="subdir1_test_1"),
        Mock(name="subdir1_test_2"),
    ]

    expected_subdir2_files = [
        Mock(name="subdir2_test_1"),
    ]

    expected_subdir1_1_files = [
        Mock(name="subdir1_1_test_1"),
        Mock(name="subdir1_1_test_2"),
    ]

    expected_subdir1_2_files = [
        Mock(name="subdir1_2_test_1"),
        Mock(name="subdir1_2_test_2"),
    ]

    expected_subdir1_web_feature_file = Mock()
    expected_subdir1_1_web_feature_file = Mock()
    expected_subdir2_web_feature_file = Mock()
    mock_web_features_file.side_effect = [
        expected_subdir1_web_feature_file,
        expected_subdir1_1_web_feature_file,
        expected_subdir2_web_feature_file,
    ]

    def fake_find_all_test_files_in_dir(root, rel_path, url_root):
        # All cases should use url_root == "/"
        if url_root != "/":
            return None
        elif (root == "repo_root" and rel_path == "."):
            return expected_root_files
        elif (root == "repo_root" and rel_path == "subdir1"):
            return expected_subdir1_files
        elif (root == "repo_root" and rel_path == os.path.join("subdir1", "subdir1_1")):
            return expected_subdir1_1_files
        elif (root == "repo_root" and rel_path == os.path.join("subdir1", "subdir1_2")):
            return expected_subdir1_2_files
        elif (root == "repo_root" and rel_path == "subdir2"):
            return expected_subdir2_files
    mock_find_all_test_files_in_dir.side_effect = fake_find_all_test_files_in_dir
    cmd_cfg = CmdConfig("repo_root", "/")
    result = WebFeaturesMap()

    map_tests_to_web_features(cmd_cfg, "", result)

    assert mock_isfile.call_count == 5
    assert mock_mapper.call_count == 5

    # Check for the constructor calls.
    # In between also assert that the run() call is executed.
    mock_mapper.assert_has_calls([
        call(expected_root_files, None),
        call().run(ANY, []),
        call(expected_subdir1_files, expected_subdir1_web_feature_file),
        call().run(ANY, []),
        call(expected_subdir1_1_files, expected_subdir1_1_web_feature_file),
        call().run(ANY, []),
        call(expected_subdir1_2_files, None),
        call().run(ANY, []),
        call(expected_subdir2_files, expected_subdir2_web_feature_file),
        call().run(ANY, []),
    ])


    # Only five times to the constructor
    assert mock_mapper.call_count == 5


def test_parser_with_path_provided_abs_path():
    parser = create_parser()
    args = parser.parse_args(["--path", f"{os.path.abspath(os.sep)}manifest-path"])
    assert args.path == f"{os.path.abspath(os.sep)}manifest-path"

def populate_test_web_features_map(web_features_map):
    web_features_map.add("grid", [
        Mock(spec=URLManifestItem, url="/grid_test1.js"),
        Mock(spec=URLManifestItem, url="/grid_test2.js"),
    ])
    web_features_map.add("avif", [Mock(spec=URLManifestItem, url="/avif_test1.js")])


def test_valid_schema():
    with open(os.path.join(os.path.dirname(__file__), '..', 'MANIFEST_SCHEMA.json'), 'r') as schema_file:
        schema_dict = json.load(schema_file)

    web_features_map = WebFeaturesMap()
    populate_test_web_features_map(web_features_map)

    with patch('builtins.open', new_callable=mock_open) as mock_file:
        write_manifest_file("test_file.json", web_features_map)
        mock_file.assert_called_once_with("test_file.json", "w")
        mock_file.return_value.write.assert_called_once_with(
            ('{"version": 1,'
            ' "data": {"grid": ["/grid_test1.js", "/grid_test2.js"], "avif": ["/avif_test1.js"]}}'))
        args = mock_file.return_value.write.call_args
        file_dict = json.loads(args[0][0])
        # Should not throw an exception
        try:
            validate(file_dict, schema_dict)
        except Exception as e:
            assert False, f"'validate' raised an exception {e}"


@pytest.mark.parametrize('main_kwargs,expected_repo_root,expected_path', [
    # No flags. All default
    (
        {},
        localpaths.repo_root,
        os.path.join(localpaths.repo_root, "WEB_FEATURES_MANIFEST.json")
    ),
    # Provide the path flag
    (
        {
            "path": os.path.join(os.sep, "test_path", "WEB_FEATURES_MANIFEST.json"),
        },
        localpaths.repo_root,
        os.path.join(os.sep, "test_path", "WEB_FEATURES_MANIFEST.json")
    ),
])
@patch("tools.web_features.manifest.map_tests_to_web_features")
@patch("tools.web_features.manifest.write_manifest_file")
def test_main(
        mock_write_manifest_file,
        mock_map_tests_to_web_features,
        main_kwargs,
        expected_repo_root,
        expected_path):

    def fake_map_tests_to_web_features(
            cmd_cfg,
            rel_dir_path,
            result,
            prev_inherited_features = []):
        populate_test_web_features_map(result)

    default_kwargs = {"url_base": "/"}
    main_kwargs.update(default_kwargs)
    mock_map_tests_to_web_features.side_effect = fake_map_tests_to_web_features
    main(**main_kwargs)
    mock_map_tests_to_web_features.assert_called_once_with(CmdConfig(repo_root=expected_repo_root, url_base="/"), "", ANY)
    mock_write_manifest_file.assert_called_once()
    args = mock_write_manifest_file.call_args
    path = args[0][0]
    file = args[0][1]
    assert path == expected_path
    assert file.to_dict() == {
        'avif': ['/avif_test1.js'],
        'grid': ['/grid_test1.js', '/grid_test2.js']}
