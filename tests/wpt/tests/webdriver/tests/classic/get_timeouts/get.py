from tests.support.asserts import assert_success


def get_timeouts(session):
    return session.transport.send(
        "GET", "session/{session_id}/timeouts".format(**vars(session)))


def test_types_default_values(session):
    response = get_timeouts(session)

    timeouts = assert_success(response)
    assert isinstance(timeouts, dict)

    assert isinstance(timeouts.get("implicit"), int)
    assert isinstance(timeouts.get("pageLoad"), int)
    assert isinstance(timeouts.get("script"), int)


def test_types_null(session):
    session.timeouts.implicit = None
    session.timeouts.page_load = None
    session.timeouts.script = None

    response = get_timeouts(session)
    timeouts = assert_success(response)

    assert timeouts == {
        "script": None,
        "implicit": None,
        "pageLoad": None,
    }


def test_custom_values(session):
    session.timeouts.implicit = 1
    session.timeouts.page_load = 200
    session.timeouts.script = 60

    response = get_timeouts(session)
    timeouts = assert_success(response)

    assert timeouts["script"] == 60000
    assert timeouts["implicit"] == 1000
    assert timeouts["pageLoad"] == 200000
