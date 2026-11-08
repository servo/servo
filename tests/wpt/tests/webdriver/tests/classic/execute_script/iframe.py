import pytest

import webdriver.protocol as protocol

from webdriver.client import WebElement
from tests.support.classic.asserts import assert_success


def execute_script(session, script, args=None):
    if args is None:
        args = []
    body = {"script": script, "args": args}

    return session.transport.send(
        "POST",
        "/session/{session_id}/execute/sync".format(**vars(session)),
        body,
        encoder=protocol.Encoder,
        decoder=protocol.Decoder,
        session=session,
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_element_argument_and_return_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<div id='target'>bar</div>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#target", all=False)

    result = execute_script(
        session,
        "return [arguments[0].id, arguments[0].tagName.toLowerCase(), arguments[0]]",
        args=[element],
    )
    actual = assert_success(result)

    assert actual[0] == "target"
    assert actual[1] == "div"
    assert isinstance(actual[2], WebElement)  # returned node round-trips...
    assert actual[2].id == element.id         # ...to the same handle.
