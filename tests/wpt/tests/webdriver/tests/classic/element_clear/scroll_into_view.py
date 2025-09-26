import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import is_element_in_viewport

from . import element_clear


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

        <div class="scroll-container">
            <input type="text" size="20" value="foo"></input>
        </div>
        """)

    element = session.find.css("input", all=False)
    response = element_clear(session, element)
    assert_success(response)

    assert is_element_in_viewport(session, element)

    assert element.property("value") == ""

