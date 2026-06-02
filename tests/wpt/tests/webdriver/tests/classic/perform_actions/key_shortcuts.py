from tests.classic.perform_actions.support.refine import get_keys
from tests.support.keys import Keys


def test_mod_a_and_backspace_deletes_all_text(session, key_reporter, key_chain, modifier_key):
    key_chain.send_keys("abc d") \
             .key_down(modifier_key) \
             .key_down("a") \
             .key_up(modifier_key) \
             .key_up("a") \
             .key_down(Keys.BACKSPACE) \
             .perform()
    assert get_keys(key_reporter) == ""


def test_mod_a_mod_c_right_mod_v_pastes_text(session, key_reporter, key_chain, modifier_key):
    initial = "abc d"
    key_chain.send_keys(initial) \
             .key_down(modifier_key) \
             .key_down("a") \
             .key_up(modifier_key) \
             .key_up("a") \
             .key_down(modifier_key) \
             .key_down("c") \
             .key_up(modifier_key) \
             .key_up("c") \
             .send_keys([Keys.RIGHT]) \
             .key_down(modifier_key) \
             .key_down("v") \
             .key_up(modifier_key) \
             .key_up("v") \
             .perform()
    assert get_keys(key_reporter) == initial * 2


def test_mod_a_mod_x_deletes_all_text(session, key_reporter, key_chain, modifier_key):
    key_chain.send_keys("abc d") \
             .key_down(modifier_key) \
             .key_down("a") \
             .key_up(modifier_key) \
             .key_up("a") \
             .key_down(modifier_key) \
             .key_down("x") \
             .key_up(modifier_key) \
             .key_up("x") \
             .perform()
    assert get_keys(key_reporter) == ""
