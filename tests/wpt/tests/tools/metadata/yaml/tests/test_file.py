# mypy: allow-untyped-defs

from ..load import load_data_to_dict
from io import StringIO

import pytest
import yaml

def test_load_data_to_dict():
    input_buffer = StringIO("""
key:
  - value1
  - value2
""")
    result = load_data_to_dict(input_buffer)
    assert result == {"key": ["value1", "value2"]}

def test_load_data_to_dict_not_dict():
    input_buffer = StringIO("""
- key: 2
""")
    with pytest.raises(ValueError):
        load_data_to_dict(input_buffer)

def test_load_data_to_dict_invalid_yaml():
    input_buffer = StringIO("""
key: 1
- test: value
""")
    with pytest.raises(yaml.parser.ParserError):
        load_data_to_dict(input_buffer)
