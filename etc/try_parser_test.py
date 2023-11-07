#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest
from try_parser import Config, Lexer, Token, TokenType

EOF = Token(TokenType.Eof, "EOF")
LPAREN = Token(TokenType.Lparen, "(")
RPAREN = Token(TokenType.Rparen, ")")
COMMA = Token(TokenType.Comma, ",")
EQ = Token(TokenType.Assign, "=")


def string(s: str) -> Token:
    return Token(TokenType.String, s)


class TestLexer(unittest.TestCase):
    def test_string(self):
        self.assertListEqual(Lexer("linux").collect(), [string("linux"), EOF])

    def test_tuple(self):
        self.assertListEqual(Lexer("linux(key=value, key2=\"val2\")").collect(),
                             [string("linux"), LPAREN,
                              string("key"), EQ, string("value"), COMMA,
                              string("key2"), EQ, string("val2"), RPAREN,
                              EOF])


class TestParser(unittest.TestCase):
    def test_string(self):
        self.assertEqual(Config("linux").toJSON(),
                         '{"fail_fast": false, "matrix": \
[{"os": "linux", "name": "Linux", "layout": "none", "profile": "release", "unit_tests": true}]}')

    def test_tuple0(self):
        conf = Config("linux()")
        self.assertEqual(conf.toJSON(),
                         '{"fail_fast": false, "matrix": \
[{"os": "linux", "name": "Linux", "layout": "none", "profile": "release", "unit_tests": true}]}')

    def test_tuple1(self):
        conf = Config("linux(profile='debug')")
        self.assertEqual(conf.matrix[0].profile, "debug")

    def test_tuple2(self):
        conf = Config("linux(profile=debug, unit-tests=false);")
        self.assertEqual(conf.matrix[0].profile, 'debug')
        self.assertEqual(conf.matrix[0].unit_tests, False)


if __name__ == "__main__":
    unittest.main()
