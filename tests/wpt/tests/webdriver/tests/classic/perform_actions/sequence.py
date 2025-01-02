from tests.classic.perform_actions.support.refine import get_events, get_keys


def test_perform_no_actions_send_no_events(session, key_reporter, key_chain):
    key_chain.perform()
    assert len(get_keys(key_reporter)) == 0
    assert len(get_events(session)) == 0
