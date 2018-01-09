from tests.support.asserts import assert_error


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}
    return session.transport.send(
        "POST",
        "/session/{session_id}/execute/sync".format(
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
        return {arr};
        """)
    assert_error(response, "javascript error")


def test_object_in_array(session):
    response = execute_script(session, """
        let obj = {};
        obj.reference = obj;
        return [obj];
        """)
    assert_error(response, "javascript error")
