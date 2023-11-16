#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json

from enum import Enum


# lexer
class TokenType(Enum):
    Illegal = 0
    Eof = 1
    #
    Comma = 2
    SemiColon = 3
    Assign = 4
    # ()
    Lparen = 5
    Rparen = 6
    #
    String = 7
    # Keyword
    FailFast = 8
    Full = 9
    Try = 10  # nop token

    def __str__(self) -> str:
        return self.name

    def __repr__(self) -> str:
        return self.name


class Token:
    m_type: TokenType
    m_lit: str

    def __init__(self, typ: TokenType, lit: str):
        self.m_type: TokenType = typ
        self.m_lit: str = lit

    def is_seperator(self) -> bool:
        return self.m_type in [TokenType.Comma, TokenType.SemiColon]

    def __str__(self) -> str:
        return str(self.m_type) + ": '" + self.m_lit + "'"

    def __repr__(self) -> str:
        return self.__str__()

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Token):
            return self.m_lit == other.m_lit and self.m_type == other.m_type
        return False


class Lexer:
    input: str = ""
    pos: int = 0
    read_pos: int = 0
    ch: str = '\0'
    _peek_token: Token = Token(TokenType.Eof, "EOF")

    def __init__(self, s: str) -> None:
        self.input = s
        self.read_char()
        self._next_token()

    def read_char(self) -> None:
        self.ch = '\0' if self.read_pos >= len(self.input) else self.input[self.read_pos]
        self.pos = self.read_pos
        self.read_pos += 1

    def skip_whitespace(self) -> None:
        while self.ch.isspace():
            self.read_char()

    def read_string(self, end: str = '"') -> str:
        pos = self.pos + 1
        while True:
            self.read_char()
            if self.ch == end or self.ch == '\0':
                break
        return self.input[pos:self.pos]

    def read_ident(self) -> str:
        pos = self.pos
        while self.ch.isalnum() or (self.ch == '-' or self.ch == '_'):
            self.read_char()
        return self.input[pos:self.pos]

    def keyword(self, s: str) -> TokenType:
        if s == "fail-fast":
            return TokenType.FailFast
        elif s == "full":
            return TokenType.Full
        elif s == "try":
            return TokenType.Try
        else:
            return TokenType.String

    def next_token(self) -> Token:
        t = self._peek_token
        self._next_token()
        return t

    def peek_token(self) -> Token:
        return self._peek_token

    def _next_token(self) -> None:
        self.skip_whitespace()
        tok: Token
        if self.ch == '=':
            tok = Token(TokenType.Assign, self.ch)
        elif self.ch == '"' or self.ch == "'":
            tok = Token(TokenType.String, self.read_string(self.ch))
        elif self.ch == '(':
            tok = Token(TokenType.Lparen, self.ch)
        elif self.ch == ')':
            tok = Token(TokenType.Rparen, self.ch)
        elif self.ch == ',':
            tok = Token(TokenType.Comma, self.ch)
        elif self.ch == ';':
            tok = Token(TokenType.SemiColon, self.ch)
        elif self.ch == '\0':
            tok = Token(TokenType.Eof, "EOF")
        else:
            c = self.ch
            if not c.isalnum():
                self.read_char()
                tok = Token(TokenType.Illegal, c)
            else:
                s = self.read_ident()
                tok = Token(self.keyword(s), s)
            self._peek_token = tok
            return

        self.read_char()
        self._peek_token = tok

    def collect(self) -> list[Token]:
        tokens = list()
        while self.peek_token().m_type != TokenType.Eof:
            tokens.append(self.next_token())
        tokens.append(self.next_token())
        return tokens


class Layout(str, Enum):
    none = 'none'
    layout2013 = '2013'
    layout2020 = '2020'
    all = 'all'

    @staticmethod
    def parse(s: str):
        s = s.lower()
        if s == 'none':
            return Layout.none
        elif s == 'all' or s == 'both':
            return Layout.all
        elif "legacy" in s or "2013" in s:
            return Layout.layout2013
        elif "2020" in s:
            return Layout.layout2020
        else:
            raise RuntimeError("Wrong layout option")


class OS(str, Enum):
    linux = 'linux'
    mac = 'mac'
    win = 'windows'

    @staticmethod
    def parse(s: str):
        s = s.lower()
        if s == 'linux':
            return OS.linux
        elif "mac" in s:
            return OS.mac
        elif "win" in s:
            return OS.win
        else:
            raise RuntimeError("Wrong OS; only `linux`, `mac` and `windows` are supported")


class JobConfig(object):
    """
    Represents one config tuple

    name(os=_, layout=_, ...)
    """

    def __init__(self, name: str, os: OS, layout: Layout = Layout.none,
                 profile: str = "release", unit_test: bool = True):
        self.os: OS = os
        self.name: str = name
        self.layout: Layout = layout
        self.profile: str = profile
        self.unit_tests: bool = unit_test

    @staticmethod
    def parse(name: str, overrides: dict[str, str]):
        job = preset(name)
        if job is None:
            job = JobConfig(name, OS.linux)
        for k, v in overrides.items():
            if k == "os":
                job.os = OS.parse(v)
            elif k == "layout":
                job.layout = Layout.parse(v)
            elif k == "profile":
                job.profile = v
            elif k == "unit-tests" or k == "unit-test":
                job.unit_tests = (v.lower() == "true")
            else:
                raise RuntimeError(f"Unknown key `{k}`; only `os`, `layout`, `profile` and `unit-tests` are supported.")
        return job


# preset from name
def preset(s: str) -> JobConfig | None:
    s = s.lower()

    if s == "linux":
        return JobConfig("Linux", OS.linux)
    elif s == "mac":
        return JobConfig("MacOS", OS.mac)
    elif s in ["win", "windows"]:
        return JobConfig("Windows", OS.win)
    elif s in ["wpt", "linux-wpt"]:
        return JobConfig("Linux WPT", OS.linux, layout=Layout.all)
    elif s in ["wpt-2013", "linux-wpt-2013"]:
        return JobConfig("Linux WPT legacy-layout", OS.linux, layout=Layout.layout2013)
    elif s in ["wpt-2020", "linux-wpt-2020"]:
        return JobConfig("Linux WPT layout-2020", OS.linux, layout=Layout.layout2020)
    elif s in ["mac-wpt", "wpt-mac"]:
        return JobConfig("MacOS WPT", OS.mac, layout=Layout.all)
    elif s == "mac-wpt-2013":
        return JobConfig("MacOS WPT legacy-layout", OS.mac, layout=Layout.layout2013)
    elif s == "mac-wpt-2020":
        return JobConfig("MacOS WPT layout-2020", OS.mac, layout=Layout.layout2020)
    else:
        return None


class Encoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, (Config, JobConfig)):
            return o.__dict__
        return json.JSONEncoder.default(self, o)


class Config(object):
    def __init__(self, s: str | None = None):
        self.fail_fast: bool = False
        self.matrix: list[JobConfig] = list()
        if s:
            self.parse(s)

    def parse(self, s: str):
        s = s.strip()
        # if no input provided default to full build
        if not s:
            s = "full"
        lex = Lexer(s)
        while lex.peek_token().m_type != TokenType.Eof:
            token = lex.next_token()
            if token.m_type == TokenType.String:
                name = token.m_lit
                overrides: dict[str, str] = dict()
                if lex.peek_token().m_type == TokenType.Lparen:
                    lex.next_token()  # skip over (
                    # read tuple
                    while (lex.peek_token().m_type not in [TokenType.Rparen, TokenType.Eof]):
                        key = lex.next_token().m_lit
                        lex.next_token()  # over =
                        val = lex.next_token().m_lit
                        overrides[key] = val
                        # last one has no comma
                        if lex.peek_token().is_seperator():
                            lex.next_token()  # over , or ;
                    lex.next_token()  # skip over )
                    if lex.peek_token().is_seperator():
                        lex.next_token()  # over , or ;
                self.matrix.append(JobConfig.parse(name, overrides))
            elif token.m_type == TokenType.FailFast:
                self.fail_fast = True
            elif token.m_type == TokenType.Full:
                self.matrix.append(JobConfig("Linux", OS.linux))
                self.matrix.append(JobConfig("MacOS", OS.mac))
                self.matrix.append(JobConfig("Windows", OS.win))
            else:
                pass

    def toJSON(self) -> str:
        return json.dumps(self, cls=Encoder)


def main():
    import sys
    conf = Config()
    conf.parse(" ".join(sys.argv[1:]))
    print(conf.toJSON())


if __name__ == "__main__":
    main()
