from tests.support.asserts import assert_error, assert_same_element, assert_success
from . import execute_async_script


def test_collection_self_reference(session):
    response = execute_async_script(session, """
        let arr = [];
        arr.push(arr);
        arguments[0](arr);
        """)
    assert_error(response, "javascript error")


def test_element_self_reference(session, inline):
    session.url = inline("<div></div>")
    div = session.find.css("div", all=False)

    response = execute_async_script(session, """
        let div = document.querySelector("div");
        div.reference = div;
        arguments[0](div);
        """)
    value = assert_success(response)
    assert_same_element(session, value, div)


def test_object_self_reference(session):
    response = execute_async_script(session, """
        let obj = {};
        obj.reference = obj;
        arguments[0](obj);
        """)
    assert_error(response, "javascript error")


def test_collection_self_reference_in_object(session):
    response = execute_async_script(session, """
        let arr = [];
        arr.push(arr);
        arguments[0]({'value': arr});
        """)
    assert_error(response, "javascript error")


def test_object_self_reference_in_collection(session):
    response = execute_async_script(session, """
        let obj = {};
        obj.reference = obj;
        arguments[0]([obj]);
        """)
    assert_error(response, "javascript error")


def test_element_self_reference_in_collection(session, inline):
    session.url = inline("<div></div>")
    divs = session.find.css("div")

    response = execute_async_script(session, """
        let div = document.querySelector("div");
        div.reference = div;
        arguments[0]([div]);
        """)
    value = assert_success(response)
    for expected, actual in zip(divs, value):
        assert_same_element(session, expected, actual)


def test_element_self_reference_in_object(session, inline):
    session.url = inline("<div></div>")
    div = session.find.css("div", all=False)

    response = execute_async_script(session, """
        let div = document.querySelector("div");
        div.reference = div;
        arguments[0]({foo: div});
        """)
    value = assert_success(response)
    assert_same_element(session, div, value["foo"])
