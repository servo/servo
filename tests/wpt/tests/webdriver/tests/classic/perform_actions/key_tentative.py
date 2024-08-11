import pytest

from tests.classic.perform_actions.support.refine import get_keys

@pytest.mark.parametrize("value",
    [
        ("\u0e01\u0e33"),
        ("🤷🏽‍♀️"),
    ])
def test_grapheme_cluster(session, key_reporter, key_chain, value):
    key_chain \
        .key_down(value) \
        .key_up(value) \
        .perform()
    assert get_keys(key_reporter) == value
