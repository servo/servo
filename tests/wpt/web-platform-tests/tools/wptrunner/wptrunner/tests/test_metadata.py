import json
import os

import pytest

from .. import metadata


def write_properties(tmp_path, data):  # type: ignore
    path = os.path.join(tmp_path, "update_properties.json")
    with open(path, "w") as f:
        json.dump(data, f)
    return path

@pytest.mark.parametrize("data",
                         [{"properties": ["prop1"]},  # type: ignore
                          {"properties": ["prop1"], "dependents": {"prop1": ["prop2"]}},
                          ])
def test_get_properties_file_valid(tmp_path, data):
    path = write_properties(tmp_path, data)
    expected = data["properties"], data.get("dependents", {})
    actual = metadata.get_properties(properties_file=path)
    assert actual == expected

@pytest.mark.parametrize("data",
                         [{},  # type: ignore
                          {"properties": "prop1"},
                          {"properties": None},
                          {"properties": ["prop1", 1]},
                          {"dependents": {"prop1": ["prop1"]}},
                          {"properties": "prop1", "dependents": ["prop1"]},
                          {"properties": "prop1", "dependents": None},
                          {"properties": "prop1", "dependents": {"prop1": ["prop2", 2]}},
                          {"properties": ["prop1"], "dependents": {"prop2": ["prop3"]}},
                          ])
def test_get_properties_file_invalid(tmp_path, data):
    path = write_properties(tmp_path, data)
    with pytest.raises(ValueError):
        metadata.get_properties(properties_file=path)


def test_extra_properties(tmp_path):  # type: ignore
    data = {"properties": ["prop1"], "dependents": {"prop1": ["prop2"]}}
    path = write_properties(tmp_path, data)
    actual = metadata.get_properties(properties_file=path, extra_properties=["prop4"])
    expected = ["prop1", "prop4"], {"prop1": ["prop2"]}
    assert actual == expected
