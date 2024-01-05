# mypy: allow-untyped-defs

from dataclasses import asdict
from ..schema import WebFeaturesFile, FeatureEntry, SpecialFileEnum, FeatureFile

import pytest
import re

@pytest.mark.parametrize(
    "input,expected_result,expected_exception_type,exception_message",
    [
        (
            {
                "features": [
                    {
                        "name": "feature1",
                        "files": ["file1", "file2"],
                    }
                ]
            },
            {
                "features": [
                    {
                        "name": "feature1",
                        "files": ["file1", "file2"],
                    }
                ]
            },
            None,
            None
        ),
        (
            {},
            None,
            ValueError,
            "Object missing required keys: ['features']"
        ),
        (
            {
                "features": [
                    {}
                ]
            },
            None,
            ValueError,
            "Object missing required keys: ['files', 'name']"
        ),
    ])
def test_web_features_file(input, expected_result, expected_exception_type, exception_message):
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=re.escape(exception_message)):
            WebFeaturesFile(input)
    else:
        assert expected_result == asdict(WebFeaturesFile(input))

@pytest.mark.parametrize(
    "input,expected_result",
    [
        (
            FeatureEntry({"name": "test1", "files":["file1"]}),
            False
        ),
        (
            FeatureEntry({"name": "test2", "files": SpecialFileEnum.RECURSIVE}),
            True
        ),
    ])
def test_does_feature_apply_recursively(input, expected_result):
    assert input.does_feature_apply_recursively() == expected_result

@pytest.mark.parametrize(
    "input_feature,input_files,expected_result",
    [
        (
            FeatureFile("*"),
            ["test.html", "TEST.HTML"],
            ["test.html", "TEST.HTML"]
        ),
        (
            FeatureFile("test.html"),
            ["test.html", "TEST.HTML"],
            ["test.html"]
        ),
        (
            FeatureFile("test*.html"),
            ["test.html", "test1.html", "TEST1.HTML", "test2.html", "test-2.html", "foo.html"],
            ["test.html", "test1.html", "test2.html", "test-2.html"]
        ),
    ])
def test_feature_file_match_files(input_feature, input_files, expected_result):
    assert input_feature.match_files(input_files) == expected_result
