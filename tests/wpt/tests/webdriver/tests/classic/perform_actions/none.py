# META: timeout=long

# Longer timeout required due to a large number of input action and event dispatch subtests.

from tests.support.classic.asserts import assert_error, assert_success
from . import perform_actions


def test_null_response_value(session, none_chain):
    response = perform_actions(session, [none_chain.pause(0).dict])
    assert_success(response, None)


def test_no_top_browsing_context(session, closed_window, none_chain):
    response = perform_actions(session, [none_chain.pause(0).dict])
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame, none_chain):
    response = perform_actions(session, [none_chain.pause(0).dict])
    assert_error(response, "no such window")
