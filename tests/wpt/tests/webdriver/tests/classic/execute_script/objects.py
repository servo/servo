from tests.support.asserts import assert_error, assert_success
from . import execute_script


def test_object(session):
    response = execute_script(session, """
        return {
            foo: 23,
            bar: true,
        };
        """)
    value = assert_success(response)
    assert value == {"foo": 23, "bar": True}


def test_nested_object(session):
    response = execute_script(session, """
        return {
            foo: {
                cheese: 23,
            },
            bar: true,
        };
        """)
    value = assert_success(response)
    assert value == {"foo": {"cheese": 23}, "bar": True}


def test_object_to_json(session):
    response = execute_script(session, """
        return {
            toJSON() {
                return ["foo", "bar"];
            }
        };
        """)
    value = assert_success(response)
    assert value == ["foo", "bar"]


def test_object_to_json_exception(session):
    response = execute_script(session, """
        return {
            toJSON() {
                throw Error("fail");
            }
        };
        """)
    assert_error(response, "javascript error")
