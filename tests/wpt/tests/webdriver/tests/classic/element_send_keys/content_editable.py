from tests.support.asserts import assert_element_has_focus


def test_sets_insertion_point_to_end(session, inline):
    session.url = inline('<div contenteditable=true>Hello,</div>')
    body = session.find.css("body", all=False)
    assert_element_has_focus(body)

    input = session.find.css("div", all=False)
    input.send_keys(' world!')
    text = session.execute_script('return arguments[0].textContent', args=[input])
    assert "Hello, world!" == text.strip()
    assert_element_has_focus(input)


def test_sets_insertion_point_to_after_last_text_node(session, inline):
    session.url = inline('<div contenteditable=true>Hel<span>lo</span>,</div>')
    input = session.find.css("div", all=False)
    input.send_keys(" world!")
    text = session.execute_script("return arguments[0].textContent", args=[input])
    assert "Hello, world!" == text.strip()


def test_no_move_caret_if_focused(session, inline):
    session.url = inline("""<div contenteditable=true>Hel<span>lo</span>,</div>
<script>document.getElementsByTagName("div")[0].focus()</script>""")
    input = session.find.css("div", all=False)
    input.send_keys("world!")
    text = session.execute_script("return arguments[0].textContent", args=[input])
    assert "world!Hello," == text.strip()
