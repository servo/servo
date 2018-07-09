import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline

def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_numbers_link(session, server_config):
    link = "/webdriver/tests/element_click/support/input.html"
    session.url = inline("<a href={url}>123456</a>".format(url=link))
    element = session.find.css("a", all=False)
    response = element_click(session, element)
    assert_success(response)
    host = server_config["browser_host"]
    port = server_config["ports"]["http"][0]

    assert session.url == "http://{host}:{port}{url}".format(host=host, port=port, url=link)


def test_multi_line_link(session, server_config):
    link = "/webdriver/tests/element_click/support/input.html"
    session.url = inline("""
        <p style="background-color: yellow; width: 50px;">
            <a href={url}>Helloooooooooooooooooooo Worlddddddddddddddd</a>
        </p>""".format(url=link))
    element = session.find.css("a", all=False)
    response = element_click(session, element)
    assert_success(response)
    host = server_config["browser_host"]
    port = server_config["ports"]["http"][0]

    assert session.url == "http://{host}:{port}{url}".format(host=host, port=port, url=link)


def test_link_unload_event(session, server_config):
    link = "/webdriver/tests/element_click/support/input.html"
    session.url = inline("""
        <body onunload="checkUnload()">
            <a href={url}>click here</a>
            <input type=checkbox>
            <script>
                function checkUnload() {{
                    document.getElementsByTagName("input")[0].checked = true;
                }}
            </script>
        </body>""".format(url=link))

    element = session.find.css("a", all=False)
    response = element_click(session, element)
    assert_success(response)

    host = server_config["browser_host"]
    port = server_config["ports"]["http"][0]
    assert session.url == "http://{host}:{port}{url}".format(host=host, port=port, url=link)

    session.back()

    element = session.find.css("input", all=False)
    response = session.execute_script("""
        let input = arguments[0];
        return input.checked;
        """, args=(element,))
    assert response is True


def test_link_hash(session):
    id = "anchor"
    session.url = inline("""
        <a href="#{url}">aaaa</a>
        <p id={id} style="margin-top: 5000vh">scroll here</p>
        """.format(url=id, id=id))
    old_url = session.url

    element = session.find.css("a", all=False)
    response = element_click(session, element)
    assert_success(response)

    new_url = session.url
    assert "{url}#{id}".format(url=old_url, id=id) == new_url

    element = session.find.css("p", all=False)
    assert session.execute_script("""
        let input = arguments[0];
        rect = input.getBoundingClientRect();
        return rect["top"] >= 0 && rect["left"] >= 0 &&
            (rect["top"] + rect["height"]) <= window.innerHeight &&
            (rect["left"] + rect["width"]) <= window.innerWidth;
            """, args=(element,)) is True


def test_link_closes_window(session, create_window):
    new_handle = create_window()
    session.window_handle = new_handle

    session.url = inline("""<a href="/webdriver/tests/element_click/support/close_window.html">asdf</a>""")
    element = session.find.css("a", all=False)
    response = element_click(session, element)
    assert_success(response)
    assert new_handle not in session.handles
