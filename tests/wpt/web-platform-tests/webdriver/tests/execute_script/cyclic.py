from tests.support.asserts import assert_error, assert_same_element, assert_success
from tests.support.inline import inline


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST", "/session/{session_id}/execute/sync".format(
            session_id=session.session_id),
        body)


def test_array(session):
    response = execute_script(session, """
        let arr = [];
        arr.push(arr);
        return arr;
        """)
    assert_error(response, "javascript error")


def test_object(session):
    response = execute_script(session, """
        let obj = {};
        obj.reference = obj;
        return obj;
        """)
    assert_error(response, "javascript error")


def test_array_in_object(session):
    response = execute_script(session, """
        let arr = [];
        arr.push(arr);
        return {'arrayValue': arr};
        """)
    assert_error(response, "javascript error")


def test_object_in_array(session):
    response = execute_script(session, """
        let obj = {};
        obj.reference = obj;
        return [obj];
        """)
    assert_error(response, "javascript error")


def test_element_in_collection(session):
    session.url = inline("<div></div>")
    divs = session.find.css("div")

    response = execute_script(session, """
        let div = document.querySelector("div");
        div.reference = div;
        return [div];
        """)
    value = assert_success(response)
    for expected, actual in zip(divs, value):
        assert_same_element(session, expected, actual)


def test_element_in_object(session):
    session.url = inline("<div></div>")
    div = session.find.css("div", all=False)

    response = execute_script(session, """
        let div = document.querySelector("div");
        div.reference = div;
        return {foo: div};
        """)
    value = assert_success(response)
    assert_same_element(session, div, value["foo"])
