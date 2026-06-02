import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success


def set_timeouts(session, timeouts):
    return session.transport.send(
        "POST", "session/{session_id}/timeouts".format(**vars(session)),
        timeouts)


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/timeouts".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session):
    timeouts = {"implicit": 10, "pageLoad": 10, "script": 10}
    response = set_timeouts(session, timeouts)
    value = assert_success(response)
    assert value is None


@pytest.mark.parametrize("value", [1, "{}", False, []])
def test_parameters_invalid(session, value):
    response = set_timeouts(session, value)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [{}, {"a": 42}])
def test_parameters_unknown_fields(session, value):
    original = session.timeouts._get()

    response = set_timeouts(session, value)
    assert_success(response)

    assert session.timeouts._get() == original


def test_script_parameter_empty_no_change(session):
    original = session.timeouts._get()

    response = set_timeouts(session, {"implicit": 100})
    assert_success(response)

    assert session.timeouts._get()["script"] == original["script"]


@pytest.mark.parametrize("typ", ["implicit", "pageLoad", "script"])
@pytest.mark.parametrize("value", [[], {}, False, "10"])
def test_value_invalid_types(session, typ, value):
    response = set_timeouts(session, {typ: value})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("typ", ["implicit", "pageLoad", "script"])
@pytest.mark.parametrize("value", [-1, 2.5, 2**53])
def test_invalid_value(session, typ, value):
    response = set_timeouts(session, {typ: value})
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("key", ["implicit", "pageLoad", "script"])
def test_value_null(session, key):
    response = set_timeouts(session, {key: None})
    assert_success(response)

    assert session.timeouts._get(key) is None


@pytest.mark.parametrize("typ", ["implicit", "pageLoad", "script"])
@pytest.mark.parametrize("value", [0, 2.0, 2**53 - 1])
def test_positive_integer(session, typ, value):
    response = set_timeouts(session, {typ: value})
    assert_success(response)

    assert session.timeouts._get(typ) == value


def test_set_all_fields(session):
    timeouts = {"implicit": 10, "pageLoad": 20, "script": 30}
    response = set_timeouts(session, timeouts)
    assert_success(response)

    assert session.timeouts.implicit == 10
    assert session.timeouts.page_load == 20
    assert session.timeouts.script == 30
