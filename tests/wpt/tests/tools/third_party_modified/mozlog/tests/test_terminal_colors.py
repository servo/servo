# encoding: utf-8

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
from io import StringIO

import pytest
from mozterm import Terminal


@pytest.fixture
def terminal():
    blessed = pytest.importorskip("blessed")

    kind = "xterm-256color"
    try:
        term = Terminal(stream=StringIO(), force_styling=True, kind=kind)
    except blessed.curses.error:
        pytest.skip("terminal '{}' not found".format(kind))

    return term


EXPECTED_DICT = {
    "log_test_status_fail": "\x1b[31mlog_test_status_fail\x1b(B\x1b[m",
    "log_process_output": "\x1b[34mlog_process_output\x1b(B\x1b[m",
    "log_test_status_pass": "\x1b[32mlog_test_status_pass\x1b(B\x1b[m",
    "log_test_status_unexpected_fail": "\x1b[31mlog_test_status_unexpected_fail\x1b(B\x1b[m",
    "log_test_status_known_intermittent": "\x1b[33mlog_test_status_known_intermittent\x1b(B\x1b[m",
    "time": "\x1b[36mtime\x1b(B\x1b[m",
    "action": "\x1b[33maction\x1b(B\x1b[m",
    "pid": "\x1b[36mpid\x1b(B\x1b[m",
    "heading": "\x1b[1m\x1b[33mheading\x1b(B\x1b[m",
    "sub_heading": "\x1b[33msub_heading\x1b(B\x1b[m",
    "error": "\x1b[31merror\x1b(B\x1b[m",
    "warning": "\x1b[33mwarning\x1b(B\x1b[m",
    "bold": "\x1b[1mbold\x1b(B\x1b[m",
    "grey": "\x1b[38;2;190;190;190mgrey\x1b(B\x1b[m",
    "normal": "\x1b[90mnormal\x1b(B\x1b[m",
    "bright_black": "\x1b[90mbright_black\x1b(B\x1b[m",
}


@pytest.mark.skipif(
    not sys.platform.startswith("win"),
    reason="Only do ANSI Escape Sequence comparisons on Windows.",
)
def test_terminal_colors(terminal):
    from mozlog.formatters.machformatter import TerminalColors, color_dict

    actual_dict = TerminalColors(terminal, color_dict)

    for key in color_dict:
        assert getattr(actual_dict, key)(key) == EXPECTED_DICT[key]


if __name__ == "__main__":
    import mozunit
    mozunit.main()
