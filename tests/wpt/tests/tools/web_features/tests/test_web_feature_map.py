# mypy: allow-untyped-defs

from unittest.mock import Mock, patch

from ...manifest.item import URLManifestItem
from ...metadata.webfeatures.schema import FeatureFile
from ..web_feature_map import WebFeaturesMap, WebFeatureToTestsDirMapper


TEST_FILES = [
    Mock(
        path="root/blob-range.any.js",
        manifest_items=Mock(
            return_value=(
                None,
                [
                    Mock(spec=URLManifestItem, url="/root/blob-range.any.html"),
                    Mock(spec=URLManifestItem, url="/root/blob-range.any.worker.html"),
                ])
        )
    ),
    Mock(
        path="root/foo-range.any.js",
        manifest_items=Mock(
            return_value=(
                None,
                [
                    Mock(spec=URLManifestItem, url="/root/foo-range.any.html"),
                    Mock(spec=URLManifestItem, url="/root/foo-range.any.worker.html"),
                ])
        )
    ),
]

def test_process_recursive_feature():
    mapper = WebFeatureToTestsDirMapper(TEST_FILES, None)
    result = WebFeaturesMap()
    inherited_features = []

    feature_entry = Mock()
    feature_entry.name = "grid"
    mapper._process_recursive_feature(inherited_features, feature_entry, result)

    assert result.to_dict() == {
        "grid": [
            "/root/blob-range.any.html",
            "/root/blob-range.any.worker.html",
            "/root/foo-range.any.html",
            "/root/foo-range.any.worker.html",
        ],
    }
    assert inherited_features == ["grid"]


def test_process_non_recursive_feature():
    feature_name = "feature1"
    feature_files = [
        FeatureFile("blob-range.any.js"),  # Matches blob-range.any.js
        FeatureFile("blob-range.html"),  # Doesn't match any test file
    ]

    mapper = WebFeatureToTestsDirMapper(TEST_FILES, None)
    result = WebFeaturesMap()

    mapper._process_non_recursive_feature(feature_name, feature_files, result)

    assert result.to_dict() == {
        "feature1": [
            "/root/blob-range.any.html",
            "/root/blob-range.any.worker.html",
        ]
    }


def test_process_inherited_features():
    mapper = WebFeatureToTestsDirMapper(TEST_FILES, None)
    result = WebFeaturesMap()
    result.add("avif", [
        Mock(spec=URLManifestItem, path="root/bar-range.any.html", url="/root/bar-range.any.html"),
        Mock(spec=URLManifestItem, path="root/bar-range.any.worker.html", url="/root/bar-range.any.worker.html"),
    ])
    inherited_features = ["avif", "grid"]

    mapper._process_inherited_features(inherited_features, result)

    assert result.to_dict() == {
        "avif": [
            "/root/bar-range.any.html",
            "/root/bar-range.any.worker.html",
            "/root/blob-range.any.html",
            "/root/blob-range.any.worker.html",
            "/root/foo-range.any.html",
            "/root/foo-range.any.worker.html",
        ],
        "grid": [
            "/root/blob-range.any.html",
            "/root/blob-range.any.worker.html",
            "/root/foo-range.any.html",
            "/root/foo-range.any.worker.html",
        ],
    }
    assert inherited_features == ["avif", "grid"]

def create_feature_entry(name, recursive=False, files=None):
    rv = Mock(does_feature_apply_recursively=Mock(return_value=recursive))
    rv.name = name
    rv.files = files
    return rv


@patch("tools.web_features.web_feature_map.WebFeatureToTestsDirMapper._process_recursive_feature")
@patch("tools.web_features.web_feature_map.WebFeatureToTestsDirMapper._process_non_recursive_feature")
@patch("tools.web_features.web_feature_map.WebFeatureToTestsDirMapper._process_inherited_features")
def test_run_with_web_feature_file(
        _process_inherited_features,
        _process_non_recursive_feature,
        _process_recursive_feature):
    feature_entry1 = create_feature_entry("feature1", True)
    feature_entry2 = create_feature_entry("feature2", files=[FeatureFile("test_file1.py")])
    mock_web_feature_file = Mock(
        features=[
            feature_entry1,
            feature_entry2,
        ])
    mapper = WebFeatureToTestsDirMapper(TEST_FILES, mock_web_feature_file)


    result = WebFeaturesMap()
    mapper.run(result, ["foo", "bar"])

    _process_recursive_feature.assert_called_once_with(
        [], feature_entry1, result
    )
    _process_non_recursive_feature.assert_called_once_with(
        "feature2", [FeatureFile("test_file1.py")], result
    )

    assert not _process_inherited_features.called

@patch("tools.web_features.web_feature_map.WebFeatureToTestsDirMapper._process_recursive_feature")
@patch("tools.web_features.web_feature_map.WebFeatureToTestsDirMapper._process_non_recursive_feature")
@patch("tools.web_features.web_feature_map.WebFeatureToTestsDirMapper._process_inherited_features")
def test_run_without_web_feature_file(
        _process_inherited_features,
        _process_non_recursive_feature,
        _process_recursive_feature):
    mapper = WebFeatureToTestsDirMapper(TEST_FILES, None)

    result = WebFeaturesMap()
    mapper.run(result, ["foo", "bar"])

    assert not _process_recursive_feature.called
    assert not _process_non_recursive_feature.called

    _process_inherited_features.assert_called_once_with(
        ["foo", "bar"], result
    )
