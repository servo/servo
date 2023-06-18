import pytest

from webdriver.error import NoSuchElementException

from tests.support.asserts import assert_success
from tests.support.helpers import wait_for_new_handle
from tests.support.sync import Poll


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_numbers_link(session, server_config, inline):
    link = "/webdriver/tests/classic/element_click/support/input.html"
    session.url = inline("<a href={url}>123456</a>".format(url=link))
    element = session.find.css("a", all=False)
    response = element_click(session, element)
    assert_success(response)
    host = server_config["browser_host"]
    port = server_config["ports"]["http"][0]

    assert session.url == "http://{host}:{port}{url}".format(host=host, port=port, url=link)


def test_multi_line_link(session, server_config, inline):
    link = "/webdriver/tests/classic/element_click/support/input.html"
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


def test_link_unload_event(session, server_config, inline):
    link = "/webdriver/tests/classic/element_click/support/input.html"
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


def test_link_hash(session, inline):
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


@pytest.mark.parametrize("target", [
    "",
    "_blank",
    "_parent",
    "_self",
    "_top",
])
def test_link_from_toplevel_context_with_target(session, inline, target):
    target_page = inline("<p id='foo'>foo</p>")

    session.url = inline("<a href='{}' target='{}'>click</a>".format(target_page, target))
    element = session.find.css("a", all=False)

    orig_handles = session.handles

    response = element_click(session, element)
    assert_success(response)

    if target == "_blank":
        session.window_handle = wait_for_new_handle(session, orig_handles)

    wait = Poll(
        session,
        timeout=5,
        ignored_exceptions=NoSuchElementException,
        message="Expected element has not been found")
    wait.until(lambda s: s.find.css("#foo"))


@pytest.mark.parametrize("target", [
    "",
    "_blank",
    "_parent",
    "_self",
    "_top",
])
def test_link_from_nested_context_with_target(session, inline, iframe, target):
    target_page = inline("<p id='foo'>foo</p>")

    session.url = inline(iframe("<a href='{}' target='{}'>click</a>".format(target_page, target)))
    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    element = session.find.css("a".format(target), all=False)

    orig_handles = session.handles

    response = element_click(session, element)
    assert_success(response)

    if target == "_blank":
        session.window_handle = wait_for_new_handle(session, orig_handles)

    # With the current browsing context removed the navigation should
    # not timeout. Switch to the target context, and wait until the expected
    # element is available.
    if target == "_parent":
        session.switch_frame("parent")
    elif target == "_top":
        session.switch_frame(None)

    wait = Poll(
        session,
        timeout=5,
        ignored_exceptions=NoSuchElementException,
        message="Expected element has not been found")
    wait.until(lambda s: s.find.css("#foo"))


# Capability needed as long as no valid certificate is available:
#   https://github.com/web-platform-tests/wpt/issues/28847
@pytest.mark.capabilities({"acceptInsecureCerts": True})
def test_link_cross_origin(session, inline, url):
    base_path = ("/webdriver/tests/support/html/subframe.html" +
                 "?pipe=header(Cross-Origin-Opener-Policy,same-origin)")
    target_page = url(base_path, protocol="https", domain="alt")

    session.url = inline("<a href='{}'>click me</a>".format(target_page), protocol="https")
    link = session.find.css("a", all=False)

    response = element_click(session, link)
    assert_success(response)

    assert session.url == target_page

    session.find.css("#delete", all=False)


def test_link_closes_window(session, inline):
    new_handle = session.new_window()
    session.window_handle = new_handle

    session.url = inline("""<a href="javascript:window.close()">Close me</a>""")
    element = session.find.css("a", all=False)

    response = element_click(session, element)
    assert_success(response)

    assert new_handle not in session.handles
