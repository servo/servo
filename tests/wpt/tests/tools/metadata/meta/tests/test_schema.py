# mypy: allow-untyped-defs

from dataclasses import asdict
from ..schema import MetaFile

import pytest
import re

@pytest.mark.parametrize(
    "input,expected_result,expected_exception_type,exception_message",
    [
        (
            {
                "spec": "spec-value",
                "suggested_reviewers": ["reviewer_1", "reviewer_2"]
            },
            {
                "spec": "spec-value",
                "suggested_reviewers": ["reviewer_1", "reviewer_2"]
            },
            None,
            None
        ),
        (
            {
                "spec": "spec-value",
            },
            {
                "spec": "spec-value",
                "suggested_reviewers": None,
            },
            None,
            None
        ),
        (
            {
                "suggested_reviewers": ["reviewer_1", "reviewer_2"]
            },
            {
                "spec": None,
                "suggested_reviewers": ["reviewer_1", "reviewer_2"],
            },
            None,
            None
        ),
        (
            {},
            {"spec": None, "suggested_reviewers": None},
            None,
            None
        ),
        (
            {
                "spec": "spec-value",
                "suggested_reviewers": ["reviewer_1", 3]
            },
            None,
            ValueError,
            "Input value ['reviewer_1', 3] does not fit one of the expected values for the union"
        ),
        (
            {
                "spec": "spec-value",
                "suggested_reviewers": ["reviewer_1", "reviewer_2"],
                "extra": "test"
            },
            None,
            ValueError,
            "Object contains invalid keys: ['extra']"
        ),
    ])
def test_meta_file(input, expected_result, expected_exception_type, exception_message):
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=re.escape(exception_message)):
            MetaFile(input)
    else:
        assert expected_result == asdict(MetaFile(input))
