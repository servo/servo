from tests.support.asserts import assert_error, assert_success
from . import execute_async_script


def test_object(session):
    response = execute_async_script(session, """
        arguments[0]({
            foo: 23,
            bar: true,
        });
        """)
    value = assert_success(response)
    assert value == {"foo": 23, "bar": True}


def test_nested_object(session):
    response = execute_async_script(session, """
        arguments[0]({
            foo: {
                cheese: 23,
            },
            bar: true,
        });
        """)
    value = assert_success(response)
    assert value == {"foo": {"cheese": 23}, "bar": True}


def test_object_to_json(session):
    response = execute_async_script(session, """
        arguments[0]({
            toJSON() {
                return ["foo", "bar"];
            }
        });
        """)
    value = assert_success(response)
    assert value == ["foo", "bar"]


def test_object_to_json_exception(session):
    response = execute_async_script(session, """
        arguments[0]({
            toJSON() {
                throw Error("fail");
            }
        });
        """)
    assert_error(response, "javascript error")
