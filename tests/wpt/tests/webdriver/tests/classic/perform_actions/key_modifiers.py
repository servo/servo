import pytest

from tests.support.keys import Keys


@pytest.mark.parametrize("modifier", [Keys.SHIFT, Keys.R_SHIFT])
def test_shift_modifier_and_non_printable_keys(session, key_reporter, key_chain, modifier):
    key_chain \
        .send_keys("foo") \
        .key_down(modifier) \
        .key_down(Keys.BACKSPACE) \
        .key_up(modifier) \
        .key_up(Keys.BACKSPACE) \
        .perform()

    assert key_reporter.property("value") == "fo"


@pytest.mark.parametrize("modifier", [Keys.SHIFT, Keys.R_SHIFT])
def test_shift_modifier_generates_capital_letters(session, key_reporter, key_chain, modifier):
    key_chain \
        .send_keys("b") \
        .key_down(modifier) \
        .key_down("c") \
        .key_up(modifier) \
        .key_up("c") \
        .key_down("d") \
        .key_up("d") \
        .key_down(modifier) \
        .key_down("e") \
        .key_up("e") \
        .key_down("f") \
        .key_up(modifier) \
        .key_up("f") \
        .perform()

    assert key_reporter.property("value") == "bCdEF"
