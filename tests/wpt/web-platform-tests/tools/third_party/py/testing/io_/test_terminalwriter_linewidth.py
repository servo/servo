# coding: utf-8
from __future__ import unicode_literals

from py._io.terminalwriter import TerminalWriter


def test_terminal_writer_line_width_init():
    tw = TerminalWriter()
    assert tw.chars_on_current_line == 0
    assert tw.width_of_current_line == 0


def test_terminal_writer_line_width_update():
    tw = TerminalWriter()
    tw.write('hello world')
    assert tw.chars_on_current_line == 11
    assert tw.width_of_current_line == 11


def test_terminal_writer_line_width_update_with_newline():
    tw = TerminalWriter()
    tw.write('hello\nworld')
    assert tw.chars_on_current_line == 5
    assert tw.width_of_current_line == 5


def test_terminal_writer_line_width_update_with_wide_text():
    tw = TerminalWriter()
    tw.write('乇乂ㄒ尺卂 ㄒ卄丨匚匚')
    assert tw.chars_on_current_line == 11
    assert tw.width_of_current_line == 21  # 5*2 + 1 + 5*2


def test_terminal_writer_line_width_update_with_wide_bytes():
    tw = TerminalWriter()
    tw.write('乇乂ㄒ尺卂 ㄒ卄丨匚匚'.encode('utf-8'))
    assert tw.chars_on_current_line == 11
    assert tw.width_of_current_line == 21


def test_terminal_writer_line_width_composed():
    tw = TerminalWriter()
    text = 'café food'
    assert len(text) == 9
    tw.write(text)
    assert tw.chars_on_current_line == 9
    assert tw.width_of_current_line == 9


def test_terminal_writer_line_width_combining():
    tw = TerminalWriter()
    text = 'café food'
    assert len(text) == 10
    tw.write(text)
    assert tw.chars_on_current_line == 10
    assert tw.width_of_current_line == 9
