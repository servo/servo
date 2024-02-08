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
import sys
from typing import Optional
import unittest
import logging

from dataclasses import dataclass
from enum import Enum, Flag, auto


class Layout(Flag):
    none = 0
    layout2013 = auto()
    layout2020 = auto()

    @staticmethod
    def all():
        return Layout.layout2013 | Layout.layout2020

    def to_string(self):
        if Layout.all() in self:
            return "all"
        elif Layout.layout2020 in self:
            return "2020"
        elif Layout.layout2013 in self:
            return "2013"
        else:
            return "none"


class Workflow(str, Enum):
    LINUX = 'linux'
    MACOS = 'macos'
    WINDOWS = 'windows'
    ANDROID = 'android'


@dataclass
class JobConfig(object):
    name: str
    workflow: Workflow = Workflow.LINUX
    wpt_layout: Layout = Layout.none
    profile: str = "release"
    unit_tests: bool = False
    wpt_tests_to_run: str = ""


def handle_preset(s: str) -> Optional[JobConfig]:
    s = s.lower()

    if s == "linux":
        return JobConfig("Linux", Workflow.LINUX, unit_tests=True)
    elif s in ["mac", "macos"]:
        return JobConfig("MacOS", Workflow.MACOS, unit_tests=True)
    elif s in ["win", "windows"]:
        return JobConfig("Windows", Workflow.WINDOWS, unit_tests=True)
    elif s in ["wpt", "linux-wpt"]:
        return JobConfig("Linux WPT", Workflow.LINUX, unit_tests=True, wpt_layout=Layout.all())
    elif s in ["wpt-2013", "linux-wpt-2013"]:
        return JobConfig("Linux WPT legacy-layout", Workflow.LINUX, wpt_layout=Layout.layout2013)
    elif s in ["wpt-2020", "linux-wpt-2020"]:
        return JobConfig("Linux WPT layout-2020", Workflow.LINUX, wpt_layout=Layout.layout2020)
    elif s in ["mac-wpt", "wpt-mac"]:
        return JobConfig("MacOS WPT", Workflow.MACOS, wpt_layout=Layout.all())
    elif s == "mac-wpt-2013":
        return JobConfig("MacOS WPT legacy-layout", Workflow.MACOS, wpt_layout=Layout.layout2013)
    elif s == "mac-wpt-2020":
        return JobConfig("MacOS WPT layout-2020", Workflow.MACOS, wpt_layout=Layout.layout2020)
    elif s == "android":
        return JobConfig("Android", Workflow.ANDROID)
    elif s == "webgpu":
        return JobConfig("WebGPU CTS", Workflow.LINUX,
                         wpt_layout=Layout.layout2020,  # reftests are mode for new layout
                         wpt_tests_to_run="_webgpu",  # run only webgpu cts
                         profile="production",  # WebGPU works to slow with debug assert
                         unit_tests=False)  # production profile does not work with unit-tests
    else:
        return None


class Encoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, (Config, JobConfig)):
            return o.__dict__
        if isinstance(o, Layout):
            return o.to_string()
        return json.JSONEncoder.default(self, o)


class Config(object):
    def __init__(self, s: Optional[str] = None):
        self.fail_fast: bool = False
        self.matrix: list[JobConfig] = list()
        if s is not None:
            self.parse(s)

    def parse(self, input: str):
        input = input.lower().strip()

        if not input:
            input = "full"

        words: list[str] = input.split(" ")

        for word in words:
            # Handle keywords.
            if word in ["fail-fast", "failfast", "fail_fast"]:
                self.fail_fast = True
                continue  # skip over keyword
            if word == "full":
                words.extend(["linux-wpt", "macos", "windows", "android"])
                continue  # skip over keyword

            preset = handle_preset(word)
            if preset is None:
                print(f"Ignoring unknown preset {word}")
            else:
                self.matrix.append(preset)

    def to_json(self, **kwargs) -> str:
        return json.dumps(self, cls=Encoder, **kwargs)


def main():
    conf = Config(" ".join(sys.argv[1:]))
    print(conf.to_json())


if __name__ == "__main__":
    main()


class TestParser(unittest.TestCase):
    def test_string(self):
        self.assertDictEqual(json.loads(Config("linux fail-fast").to_json()),
                             {'fail_fast': True,
                              'matrix': [{
                                  'name': 'Linux',
                                  'profile': 'release',
                                  'unit_tests': True,
                                  'workflow': 'linux',
                                  'wpt_layout': 'none',
                                  'wpt_tests_to_run': ''
                              }]
                              })

    def test_empty(self):
        self.assertDictEqual(json.loads(Config("").to_json()),
                             {"fail_fast": False, "matrix": [
                              {
                                  "name": "Linux WPT",
                                  "workflow": "linux",
                                  "wpt_layout": "all",
                                  "profile": "release",
                                  "unit_tests": True,
                                  "wpt_tests_to_run": ""
                              },
                              {
                                  "name": "MacOS",
                                  "workflow": "macos",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": True,
                                  "wpt_tests_to_run": ""
                              },
                              {
                                  "name": "Windows",
                                  "workflow": "windows",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": True,
                                  "wpt_tests_to_run": ""
                              },
                              {
                                  "name": "Android",
                                  "workflow": "android",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": False,
                                  "wpt_tests_to_run": ""
                              }
                              ]})

    def test_full(self):
        self.assertDictEqual(json.loads(Config("linux-wpt macos windows android").to_json()),
                             json.loads(Config("").to_json()))


def run_tests():
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    suite = unittest.TestLoader().loadTestsFromTestCase(TestParser)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
