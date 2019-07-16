from tests.support.asserts import assert_same_element, assert_success
from tests.support.inline import inline


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}
    return session.transport.send(
        "POST", "/session/{session_id}/execute/sync".format(**vars(session)),
        body)


def test_content_attribute(session):
    session.url = inline("<input value=foobar>")
    response = execute_script(session, """
        const input = document.querySelector("input");
        return input.value;
        """)
    assert_success(response, "foobar")


def test_idl_attribute(session):
    session.url = inline("""
        <input>
        <script>
        const input = document.querySelector("input");
        input.value = "foobar";
        </script>
        """)
    response = execute_script(session, """
        const input = document.querySelector("input");
        return input.value;
        """)
    assert_success(response, "foobar")


def test_idl_attribute_element(session):
    session.url = inline("""
        <p>foo
        <p>bar

        <script>
        const elements = document.querySelectorAll("p");
        let foo = elements[0];
        let bar = elements[1];
        foo.bar = bar;
        </script>
        """)
    _foo, bar = session.find.css("p")
    response = execute_script(session, """
        const foo = document.querySelector("p");
        return foo.bar;
        """)
    value = assert_success(response)
    assert_same_element(session, bar, value)


def test_script_defining_property(session):
    session.url = inline("<input>")
    execute_script(session, """
        const input = document.querySelector("input");
        input.foobar = "foobar";
        """)
    response = execute_script(session, """
        const input = document.querySelector("input");
        return input.foobar;
        """)
    assert_success(response, "foobar")
