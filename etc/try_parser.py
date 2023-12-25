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
from typing import Any, Tuple
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "third_party", "ply"))
from ply.lex import lex  # noqa: E402
from ply.yacc import yacc  # noqa: E402


tokens = ("STRING",)

literals = ["[", "]", "=", ","]

t_ignore = " \t\r"


def t_STRING(t):
    # r"""('.*'|".*"|[a-zA-Z_0-9\-]+)"""
    r"""\"([^\\"]+|\\"|\\\\)"|'([^\\']+|\\'|\\\\)'|[a-zA-Z_0-9\-]+"""
    if t.value[0] in "\"'":
        t.value = t.value[1:-1].encode().decode("unicode_escape")
    return t


def t_ignore_newline(t):
    r"\n+"
    t.lexer.lineno += t.value.count("\n")


def t_error(t):
    print(f"Illegal character {t.value!r} at line {t.lineno}")
    t.lexer.skip(1)


# --------------- Parser ---------------


def p_config(p):
    """
    config : job_config
            | job_config ',' config
            | job_config config
    """
    if len(p) == 2:
        p[0] = [p[1]]
    elif len(p) == 4:
        p[0] = [p[1]] + p[3]
    else:
        p[0] = [p[1]] + p[2]


def p_job_config(p):
    """
    job_config : STRING
               | STRING '[' pairs ']'
    """
    if len(p) == 2:
        if p[1] in ["full", "try"]:  # keywords
            p[0] = [
                JobConfig("Linux", OS.linux),
                JobConfig("MacOS", OS.mac),
                JobConfig("Windows", OS.win)
            ]
        elif p[1] in ["fail_fast", "fail-fast"]:  # marker keywords
            # are handled before this parser was run
            p[0] = []
        else:  # single string without config
            p[0] = [JobConfig.parse(p[1])]
    else:
        p[0] = [JobConfig.parse(p[1], p[3])]


def p_pairs(p):
    """
    pairs : pair
          | pair ',' pairs
    """
    # join all pairs into array
    if len(p) == 2:
        p[0] = [p[1]]
    else:
        p[0] = [p[1]] + p[3]


def p_pair(p):
    "pair : STRING '=' STRING"
    p[0] = parse_kv(p[1], p[3])


def p_error(p):
    raise RuntimeError(f"Syntax error at {p.value!r} at line {p.lineno}")


lexer = lex()
parser = yacc(write_tables=False, debug=False)

# ---------------- Real Stuff -----------------


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
            return None


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
            return None


def parse_kv(k: str, v: str) -> Tuple[str, Any]:
    kk = k.lower()
    if kk == "os":
        val = OS.parse(v)
    elif kk in ["layout", "wpt-layout"]:
        kk = "wpt_layout"
        val = Layout.parse(v)
    elif kk in ["unit-tests", "unit-test"]:
        kk = "unit_tests"
        val = (v.lower() == "true")
    elif kk in ["wpt", "wpt_tests_to_run", "wpt-tests-to-run"]:
        kk = "wpt_tests_to_run"
        val = v
    elif kk in ["profile", "name"]:
        val = v
    else:
        raise RuntimeError(f"`{k}` is unknown option.")

    if val is None:
        raise RuntimeError(f"`{v}` is wrong value for `{k}`.")

    return (kk, val)


class JobConfig(object):
    """
    Represents one config tuple

    name[os=_, layout=_, ...]
    """

    def __init__(self, name: str, os: OS = OS.linux, layout: Layout = Layout.none,
                 profile: str = "release", unit_test: bool = True, wpt: str = ""):
        self.os: OS = os
        self.name: str = name
        self.wpt_layout: Layout = layout
        self.profile: str = profile
        self.unit_tests: bool = unit_test
        self.wpt_tests_to_run: str = wpt

    @staticmethod
    def parse(name: str, overrides: list[Tuple[str, str]] = []):
        job = preset(name)

        if job is None:  # job name is not a preset
            job = JobConfig(name)

        # apply all overrides on job
        for (k, v) in overrides:
            job.__dict__[k] = v

        return job


def preset(s: str) -> JobConfig | None:
    """returns JobConfig for a preset if exists"""

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
    elif s == "webgpu":
        return JobConfig("WebGPU CTS", OS.linux, layout=Layout.layout2020, wpt="_webgpu",
                         profile="production",  # WebGPU works to slow with debug assert
                         unit_test=False)  # production profile does not work with unit-tests
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
        # handle marker keyword
        if "fail_fast" in s or "fail-fast" in s:
            self.fail_fast = True
        # if no input provided default to full build
        if not s:
            s = "full"

        ast = parser.parse(s)
        # flatten
        self.matrix = [item for row in ast for item in row]

    def toJSON(self) -> str:
        return json.dumps(self, cls=Encoder)


def main():
    conf = Config(" ".join(sys.argv[1:]))
    print(conf.toJSON())


if __name__ == "__main__":
    main()
