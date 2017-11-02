import pytest

from tests.support.inline import inline


def test_sets_insertion_point_to_end(session):
    session.url = inline('<div contenteditable=true>Hello,</div>')
    input = session.find.css("div", all=False)
    input.send_keys(' world!')
    text = session.execute_script('return arguments[0].innerText', args=[input])
    assert "Hello, world!" == text.strip()


# 12. Let current text length be the element's length.
#
# 13. Set the text insertion caret using set selection range using current
#     text length for both the start and end parameters.
def test_sets_insertion_point_to_after_last_text_node(session):
    session.url = inline('<div contenteditable=true>Hel<span>lo</span>,</div>')
    input = session.find.css("div", all=False)
    input.send_keys(" world!")
    text = session.execute_script("return arguments[0].innerText", args=[input])
    assert "Hello, world!" == text.strip()
