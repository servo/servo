#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations

import json
import sys
from typing import ClassVar, List, Optional
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
    LINUX = "linux"
    MACOS = "macos"
    WINDOWS = "windows"
    ANDROID = "android"
    OHOS = "ohos"
    LINT = "lint"


@dataclass
class JobConfig(object):
    name: str
    workflow: Workflow = Workflow.LINUX
    wpt_layout: Layout = Layout.none
    profile: str = "release"
    unit_tests: bool = False
    bencher: bool = False
    wpt_args: str = ""
    # These are the fields that must match in between two JobConfigs for them to be able to be
    # merged. If you modify any of the fields above, make sure to update this line as well.
    merge_compatibility_fields: ClassVar[List[str]] = ['workflow', 'profile', 'wpt_args']

    def merge(self, other: JobConfig) -> bool:
        """Try to merge another job with this job. Returns True if merging is successful
           or False if not. If merging is successful this job will be modified."""
        for field in self.merge_compatibility_fields:
            if getattr(self, field) != getattr(other, field):
                return False

        self.wpt_layout |= other.wpt_layout
        self.unit_tests |= other.unit_tests
        self.bencher |= other.bencher
        common = min([self.name, other.name], key=len)
        p1 = self.name.strip(common).strip()
        p2 = other.name.strip(common).strip()
        self.name = common.strip()
        if p1:
            self.name += f" {p1}"
        if p2:
            self.name += f" {p2}"
        return True


def handle_preset(s: str) -> Optional[JobConfig]:
    s = s.lower()

    if s == "linux":
        return JobConfig("Linux", Workflow.LINUX, unit_tests=True)
    elif s in ["perf", "linux-perf", "bencher"]:
        return JobConfig("Linux perf", Workflow.LINUX, bencher=True)
    elif s == "linux-prod-perf":
        return JobConfig("Linux prod perf", Workflow.LINUX, bencher=True, profile="production")
    elif s in ["mac", "macos"]:
        return JobConfig("MacOS", Workflow.MACOS, unit_tests=True)
    elif s == "macos-perf":
        return JobConfig("MacOS", Workflow.MACOS, bencher=True)
    elif s == "macos-prod-perf":
        return JobConfig("MacOS", Workflow.MACOS, bencher=True, profile="production")
    elif s in ["win", "windows"]:
        return JobConfig("Windows", Workflow.WINDOWS, unit_tests=True)
    elif s == "windows-perf":
        return JobConfig("Windows", Workflow.WINDOWS, bencher=True)
    elif s == "windows-prod-perf":
        return JobConfig("Windows", Workflow.WINDOWS, bencher=True, profile="production")
    elif s in ["wpt-2013", "linux-wpt-2013"]:
        return JobConfig("Linux WPT", Workflow.LINUX, wpt_layout=Layout.layout2013)
    elif s in ["wpt-2020", "linux-wpt-2020", "wpt", "linux-wpt"]:
        return JobConfig("Linux WPT", Workflow.LINUX, wpt_layout=Layout.layout2020)
    elif s in ["mac-wpt", "wpt-mac"]:
        return JobConfig("MacOS WPT", Workflow.MACOS, wpt_layout=Layout.all())
    elif s == "mac-wpt-2013":
        return JobConfig("MacOS WPT", Workflow.MACOS, wpt_layout=Layout.layout2013)
    elif s == "mac-wpt-2020":
        return JobConfig("MacOS WPT", Workflow.MACOS, wpt_layout=Layout.layout2020)
    elif s == "android":
        return JobConfig("Android", Workflow.ANDROID)
    elif s == "android-perf":
        return JobConfig("Android", Workflow.ANDROID, bencher=True)
    elif s == "android-prod-perf":
        return JobConfig("Android", Workflow.ANDROID, bencher=True, profile="production")
    elif s in ["ohos", "openharmony"]:
        return JobConfig("OpenHarmony", Workflow.OHOS)
    elif s == "ohos-perf":
        return JobConfig("OpenHarmony", Workflow.OHOS, bencher=True)
    elif s == "ohos-prod-perf":
        return JobConfig("OpenHarmony", Workflow.OHOS, bencher=True, profile="production")
    elif s == "webgpu":
        return JobConfig("WebGPU CTS", Workflow.LINUX,
                         wpt_layout=Layout.layout2020,  # reftests are mode for new layout
                         wpt_args="_webgpu",  # run only webgpu cts
                         profile="production",  # WebGPU works to slow with debug assert
                         unit_tests=False)  # production profile does not work with unit-tests
    elif s in ["lint", "tidy"]:
        return JobConfig("Lint", Workflow.LINT)
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
                words.extend(["linux", "linux-wpt", "linux-perf", "linux-prod-perf"])
                words.extend(["macos", "macos-perf", "macos-prod-perf"])
                words.extend(["windows", "windows-perf", "windows-prod-perf"])
                words.extend(["android", "android-perf"])
                words.extend(["ohos", "ohos-perf", "ohos-prod-perf"])
                words.extend(["lint"])
                continue  # skip over keyword

            job = handle_preset(word)
            if job is None:
                print(f"Ignoring unknown preset {word}")
            else:
                self.add_or_merge_job_to_matrix(job)

    def add_or_merge_job_to_matrix(self, job: JobConfig):
        for existing_job in self.matrix:
            if existing_job.merge(job):
                return
        self.matrix.append(job)

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
                                  'bencher': False,
                                  'name': 'Linux',
                                  'profile': 'release',
                                  'unit_tests': True,
                                  'workflow': 'linux',
                                  'wpt_layout': 'none',
                                  'wpt_args': ''
                              }]
                              })

    def test_empty(self):
        self.assertDictEqual(json.loads(Config("").to_json()),
                             {"fail_fast": False, "matrix": [
                              {
                                  "name": "Linux WPT perf",
                                  "workflow": "linux",
                                  "wpt_layout": "2020",
                                  "profile": "release",
                                  "unit_tests": True,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "Linux prod perf",
                                  "workflow": "linux",
                                  "wpt_layout": "none",
                                  "profile": "production",
                                  "unit_tests": False,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "MacOS",
                                  "workflow": "macos",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": True,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "MacOS",
                                  "workflow": "macos",
                                  "wpt_layout": "none",
                                  "profile": "production",
                                  "unit_tests": False,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "Windows",
                                  "workflow": "windows",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": True,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "Windows",
                                  "workflow": "windows",
                                  "wpt_layout": "none",
                                  "profile": "production",
                                  "unit_tests": False,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "Android",
                                  "workflow": "android",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": False,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "OpenHarmony",
                                  "workflow": "ohos",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": False,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "OpenHarmony",
                                  "workflow": "ohos",
                                  "wpt_layout": "none",
                                  "profile": "production",
                                  "unit_tests": False,
                                  'bencher': True,
                                  "wpt_args": ""
                              },
                              {
                                  "name": "Lint",
                                  "workflow": "lint",
                                  "wpt_layout": "none",
                                  "profile": "release",
                                  "unit_tests": False,
                                  'bencher': False,
                                  "wpt_args": ""}
                              ]})

    def test_job_merging(self):
        self.assertDictEqual(json.loads(Config("wpt-2020 wpt-2013").to_json()),
                             {'fail_fast': False,
                              'matrix': [{
                                  'bencher': False,
                                  'name': 'Linux WPT',
                                  'profile': 'release',
                                  'unit_tests': False,
                                  'workflow': 'linux',
                                  'wpt_layout': 'all',
                                  'wpt_args': ''
                              }]
                              })

        a = JobConfig("Linux", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux", Workflow.LINUX, unit_tests=False)
        self.assertTrue(a.merge(b), "Should merge jobs that have different unit test configurations.")
        self.assertEqual(a, JobConfig("Linux", Workflow.LINUX, unit_tests=True))

        a = handle_preset("linux")
        b = handle_preset("linux-wpt")
        self.assertTrue(a.merge(b), "Should merge jobs that have different unit test configurations.")
        self.assertEqual(a, JobConfig("Linux WPT", Workflow.LINUX, unit_tests=True, wpt_layout=Layout.layout2020))

        a = JobConfig("Linux", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Mac", Workflow.MACOS, unit_tests=True)
        self.assertFalse(a.merge(b), "Should not merge jobs with different workflows.")
        self.assertEqual(a, JobConfig("Linux", Workflow.LINUX, unit_tests=True))

        a = JobConfig("Linux", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux", Workflow.LINUX, unit_tests=True, profile="production")
        self.assertFalse(a.merge(b), "Should not merge jobs with different profiles.")
        self.assertEqual(a, JobConfig("Linux", Workflow.LINUX, unit_tests=True))

        a = JobConfig("Linux", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux", Workflow.LINUX, unit_tests=True, wpt_args="/css")
        self.assertFalse(a.merge(b), "Should not merge jobs that run different WPT tests.")
        self.assertEqual(a, JobConfig("Linux", Workflow.LINUX, unit_tests=True))

    def test_full(self):
        self.assertDictEqual(json.loads(Config("full").to_json()),
                             json.loads(Config("").to_json()))


def run_tests():
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    suite = unittest.TestLoader().loadTestsFromTestCase(TestParser)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
