from tests.support.asserts import assert_success


def get_timeouts(session):
    return session.transport.send(
        "GET", "session/{session_id}/timeouts".format(**vars(session)))


def test_get_timeouts(session):
    response = get_timeouts(session)

    assert_success(response)
    assert "value" in response.body
    assert isinstance(response.body["value"], dict)

    value = response.body["value"]
    assert "script" in value
    assert "implicit" in value
    assert "pageLoad" in value

    assert isinstance(value["script"], int)
    assert isinstance(value["implicit"], int)
    assert isinstance(value["pageLoad"], int)


def test_get_default_timeouts(session):
    response = get_timeouts(session)

    assert_success(response)
    assert response.body["value"]["script"] == 30000
    assert response.body["value"]["implicit"] == 0
    assert response.body["value"]["pageLoad"] == 300000


def test_get_new_timeouts(session):
    session.timeouts.script = 60
    session.timeouts.implicit = 1
    session.timeouts.page_load = 200
    response = get_timeouts(session)
    assert_success(response)
    assert response.body["value"]["script"] == 60000
    assert response.body["value"]["implicit"] == 1000
    assert response.body["value"]["pageLoad"] == 200000
