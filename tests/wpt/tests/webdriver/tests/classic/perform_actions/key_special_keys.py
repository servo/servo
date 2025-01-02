import pytest

from webdriver import error

from tests.classic.perform_actions.support.refine import get_keys


@pytest.mark.parametrize("value", [
    (u"\U0001F604"),
    (u"\U0001F60D"),
    (u"\u0BA8\u0BBF"),
    (u"\u1100\u1161\u11A8"),
])
def test_codepoint_keys_behave_correctly(session, key_reporter, key_chain, value):
    # Not using key_chain.send_keys() because we always want to treat value as
    # one character here. `len(value)` varies by platform for non-BMP characters,
    # so we don't want to iterate over value.
    key_chain \
        .key_down(value) \
        .key_up(value) \
        .perform()

    # events sent by major browsers are inconsistent so only check key value
    assert get_keys(key_reporter) == value


@pytest.mark.parametrize("value", [
    (u"fa"),
    (u"\u0BA8\u0BBFb"),
    (u"\u0BA8\u0BBF\u0BA8"),
    (u"\u1100\u1161\u11A8c")
])
def test_invalid_multiple_codepoint_keys_fail(session, key_reporter, key_chain, value):
    with pytest.raises(error.InvalidArgumentException):
        key_chain \
            .key_down(value) \
            .key_up(value) \
            .perform()
