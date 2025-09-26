import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import is_element_in_viewport

from . import take_element_screenshot


def test_scroll_into_view(session, inline):
    session.url = inline("""
        <style>
            .scroll-container {
                display: block;
                overflow-y: scroll;
                scroll-behavior: smooth;
                height: 200px;
            }

            input {
                margin-top: 2000vh;
            }
        </style>

        <input id="reference" type="text" size="20" value="foo"></input>
        <div class="scroll-container">
            <input id="target" type="text" size="20" value="foo"></input>
        </div>
        """)

    reference = session.find.css("#reference", all=False)
    response = take_element_screenshot(session, reference.id)
    reference_screenshot = assert_success(response)

    element = session.find.css("#target", all=False)
    response = take_element_screenshot(session, element.id)
    screenshot = assert_success(response)

    assert is_element_in_viewport(session, element)

    assert screenshot == reference_screenshot
