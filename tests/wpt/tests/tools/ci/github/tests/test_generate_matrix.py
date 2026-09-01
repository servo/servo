from typing import Optional

import pytest
from tools.ci.github.generate_matrix import get_matrix


@pytest.mark.parametrize("chunks", [0, 1, 2, 3])
def test_chunk_count_and_numbering(chunks: int) -> None:
    includes = get_matrix(
        "any",
        {
            "defaults": {"timeout": 120},
            "test_types": {"testharness": {"chunks": chunks, "timeout": None}},
        },
    )
    assert includes == [
        {
            "test-type": "testharness",
            "current-chunk": i,
            "total-chunks": chunks,
            "timeout-minutes": 120,
        }
        for i in range(1, chunks + 1)
    ]


@pytest.mark.parametrize(
    "per_type_timeout,default_timeout,expected",
    [
        (None, 180, 180),
        (240, 180, 240),
        (120, 180, 120),
    ],
)
def test_timeout(per_type_timeout: Optional[int], default_timeout: int, expected: int) -> None:
    includes = get_matrix(
        "any",
        {
            "defaults": {"timeout": default_timeout},
            "test_types": {"testharness": {"chunks": 1, "timeout": per_type_timeout}},
        },
    )
    assert includes == [
        {
            "test-type": "testharness",
            "current-chunk": 1,
            "total-chunks": 1,
            "timeout-minutes": expected,
        }
    ]


@pytest.mark.parametrize(
    "browser,expected_chunks",
    [
        ("firefox", 2),
        ("chrome", 1),
    ],
)
def test_browser_conditional_chunks(browser: str, expected_chunks: int) -> None:
    includes = get_matrix(
        browser,
        {
            "defaults": {"timeout": 120},
            "test_types": {
                "testharness": {
                    "chunks": {"$switch": {'browser == "firefox"': 2, "$default": 1}},
                    "timeout": None,
                },
            },
        },
    )
    assert includes == [
        {
            "test-type": "testharness",
            "current-chunk": i,
            "total-chunks": expected_chunks,
            "timeout-minutes": 120,
        }
        for i in range(1, expected_chunks + 1)
    ]
