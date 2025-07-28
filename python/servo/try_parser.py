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
from enum import Enum


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
    wpt: bool = False
    profile: str = "release"
    unit_tests: bool = False
    build_libservo: bool = False
    bencher: bool = False
    build_args: str = ""
    wpt_args: str = ""
    number_of_wpt_chunks: int = 20
    # These are the fields that must match in between two JobConfigs for them to be able to be
    # merged. If you modify any of the fields above, make sure to update this line as well.
    merge_compatibility_fields: ClassVar[List[str]] = ["workflow", "profile", "wpt_args", "build_args"]

    def merge(self, other: JobConfig) -> bool:
        """Try to merge another job with this job. Returns True if merging is successful
        or False if not. If merging is successful this job will be modified."""
        for field in self.merge_compatibility_fields:
            if getattr(self, field) != getattr(other, field):
                return False

        self.wpt |= other.wpt
        self.unit_tests |= other.unit_tests
        self.build_libservo |= other.build_libservo
        self.bencher |= other.bencher
        self.number_of_wpt_chunks = max(self.number_of_wpt_chunks, other.number_of_wpt_chunks)
        self.update_name()
        return True

    def update_name(self) -> None:
        if self.workflow is Workflow.LINUX:
            self.name = "Linux"
        elif self.workflow is Workflow.MACOS:
            self.name = "MacOS"
        elif self.workflow is Workflow.WINDOWS:
            self.name = "Windows"
        elif self.workflow is Workflow.ANDROID:
            self.name = "Android"
        elif self.workflow is Workflow.OHOS:
            self.name = "OpenHarmony"
        modifier = []
        if self.profile != "release":
            modifier.append(self.profile.title())
        if self.unit_tests:
            modifier.append("Unit Tests")
        if self.build_libservo:
            modifier.append("Build libservo")
        if self.wpt:
            modifier.append("WPT")
        if self.bencher:
            modifier.append("Bencher")
        if modifier:
            self.name += " (" + ", ".join(modifier) + ")"


def handle_preset(s: str) -> Optional[JobConfig]:
    s = s.lower()

    if any(word in s for word in ["linux"]):
        return JobConfig("Linux", Workflow.LINUX)
    elif any(word in s for word in ["mac", "macos"]):
        return JobConfig("MacOS", Workflow.MACOS)
    elif any(word in s for word in ["win", "windows"]):
        return JobConfig("Windows", Workflow.WINDOWS)
    elif any(word in s for word in ["android"]):
        return JobConfig("Android", Workflow.ANDROID)
    elif any(word in s for word in ["ohos", "openharmony"]):
        return JobConfig("OpenHarmony", Workflow.OHOS)
    elif any(word in s for word in ["webgpu"]):
        return JobConfig(
            "WebGPU CTS",
            Workflow.LINUX,
            wpt=True,
            wpt_args="_webgpu",  # run only webgpu cts
            profile="production",  # WebGPU works to slow with debug assert
            unit_tests=False,
        )  # production profile does not work with unit-tests
    elif any(word in s for word in ["webdriver", "wd"]):
        return JobConfig(
            "WebDriver",
            Workflow.LINUX,
            wpt=True,
            wpt_args=" ".join(
                [
                    "./tests/wpt/tests/webdriver/tests/classic/",
                    "--product servodriver",
                    "--headless",
                    "--processes 1",
                ]
            ),
            unit_tests=False,
            number_of_wpt_chunks=2,
        )
    elif any(word in s for word in ["vello-cpu", "vello_cpu"]):
        return JobConfig(
            "Vello-CPU WPT",
            Workflow.LINUX,
            wpt=True,
            wpt_args=" ".join(
                [
                    "--subsuite-file ./tests/wpt/vello_cpu_canvas_subsuite.json",
                    "--subsuite vello_cpu_canvas",
                ]
            ),
            build_args="--features 'vello_cpu'",
        )
    elif any(word in s for word in ["vello"]):
        return JobConfig(
            "Vello WPT",
            Workflow.LINUX,
            wpt=True,
            wpt_args=" ".join(
                [
                    "--subsuite-file ./tests/wpt/vello_canvas_subsuite.json",
                    "--subsuite vello_canvas",
                    "--processes 1",
                ]
            ),
            build_args="--features 'vello'",
        )
    elif any(word in s for word in ["lint", "tidy"]):
        return JobConfig("Lint", Workflow.LINT)
    else:
        return None


def handle_modifier(config: Optional[JobConfig], s: str) -> Optional[JobConfig]:
    if config is None:
        return None
    s = s.lower()
    if "unit-tests" in s:
        config.unit_tests = True
    if "build-libservo" in s:
        config.build_libservo = True
    if "production" in s:
        config.profile = "production"
    if "bencher" in s:
        config.bencher = True
    elif "wpt" in s:
        config.wpt = True
    config.update_name()
    return config


class Encoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, (Config, JobConfig)):
            return o.__dict__
        return json.JSONEncoder.default(self, o)


class Config(object):
    def __init__(self, s: Optional[str] = None) -> None:
        self.fail_fast: bool = False
        self.matrix: list[JobConfig] = list()
        if s is not None:
            self.parse(s)

    def parse(self, input: str) -> None:
        input = input.lower().strip()

        if not input:
            input = "full"

        words: list[str] = input.split(" ")

        for word in words:
            # Handle keywords.
            if word in ["fail-fast", "failfast", "fail_fast"]:
                self.fail_fast = True
                continue  # skip over keyword
            if word == "wpt":
                words.extend(["linux-wpt"])
                continue  # skip over keyword
            if word == "full":
                words.extend(["linux-unit-tests", "linux-wpt", "linux-bencher"])
                words.extend(["macos-unit-tests", "windows-unit-tests", "android", "ohos", "lint"])
                continue  # skip over keyword
            if word == "bencher":
                words.extend(["linux-bencher", "macos-bencher", "windows-bencher", "android-bencher", "ohos-bencher"])
                continue  # skip over keyword
            if word == "production-bencher":
                words.extend(["linux-production-bencher", "macos-production-bencher", "windows-production-bencher"])
                words.extend(["ohos-production-bencher"])
                continue  # skip over keyword
            job = handle_preset(word)
            job = handle_modifier(job, word)
            if job is None:
                print(f"Ignoring unknown preset {word}")
            else:
                self.add_or_merge_job_to_matrix(job)

    def add_or_merge_job_to_matrix(self, job: JobConfig) -> None:
        for existing_job in self.matrix:
            if existing_job.merge(job):
                return
        self.matrix.append(job)

    def to_json(self, **kwargs) -> str:
        return json.dumps(self, cls=Encoder, **kwargs)


def main() -> None:
    conf = Config(" ".join(sys.argv[1:]))
    print(conf.to_json())


if __name__ == "__main__":
    main()


class TestParser(unittest.TestCase):
    def test_string(self) -> None:
        self.assertDictEqual(
            json.loads(Config("linux-unit-tests fail-fast").to_json()),
            {
                "fail_fast": True,
                "matrix": [
                    {
                        "bencher": False,
                        "name": "Linux (Unit Tests)",
                        "number_of_wpt_chunks": 20,
                        "profile": "release",
                        "unit_tests": True,
                        "build_libservo": False,
                        "workflow": "linux",
                        "wpt": False,
                        "wpt_args": "",
                        "build_args": "",
                    }
                ],
            },
        )

    def test_empty(self) -> None:
        self.assertDictEqual(
            json.loads(Config("").to_json()),
            {
                "fail_fast": False,
                "matrix": [
                    {
                        "name": "Linux (Unit Tests, WPT, Bencher)",
                        "number_of_wpt_chunks": 20,
                        "workflow": "linux",
                        "wpt": True,
                        "profile": "release",
                        "unit_tests": True,
                        "build_libservo": False,
                        "bencher": True,
                        "wpt_args": "",
                        "build_args": "",
                    },
                    {
                        "name": "MacOS (Unit Tests)",
                        "number_of_wpt_chunks": 20,
                        "workflow": "macos",
                        "wpt": False,
                        "profile": "release",
                        "unit_tests": True,
                        "build_libservo": False,
                        "bencher": False,
                        "wpt_args": "",
                        "build_args": "",
                    },
                    {
                        "name": "Windows (Unit Tests)",
                        "number_of_wpt_chunks": 20,
                        "workflow": "windows",
                        "wpt": False,
                        "profile": "release",
                        "unit_tests": True,
                        "build_libservo": False,
                        "bencher": False,
                        "wpt_args": "",
                        "build_args": "",
                    },
                    {
                        "name": "Android",
                        "number_of_wpt_chunks": 20,
                        "workflow": "android",
                        "wpt": False,
                        "profile": "release",
                        "unit_tests": False,
                        "build_libservo": False,
                        "bencher": False,
                        "wpt_args": "",
                        "build_args": "",
                    },
                    {
                        "name": "OpenHarmony",
                        "number_of_wpt_chunks": 20,
                        "workflow": "ohos",
                        "wpt": False,
                        "profile": "release",
                        "unit_tests": False,
                        "build_libservo": False,
                        "bencher": False,
                        "wpt_args": "",
                        "build_args": "",
                    },
                    {
                        "name": "Lint",
                        "number_of_wpt_chunks": 20,
                        "workflow": "lint",
                        "wpt": False,
                        "profile": "release",
                        "unit_tests": False,
                        "build_libservo": False,
                        "bencher": False,
                        "wpt_args": "",
                        "build_args": "",
                    },
                ],
            },
        )

    def test_job_merging(self) -> None:
        self.assertDictEqual(
            json.loads(Config("linux-wpt").to_json()),
            {
                "fail_fast": False,
                "matrix": [
                    {
                        "bencher": False,
                        "name": "Linux (WPT)",
                        "number_of_wpt_chunks": 20,
                        "profile": "release",
                        "unit_tests": False,
                        "build_libservo": False,
                        "workflow": "linux",
                        "wpt": True,
                        "wpt_args": "",
                        "build_args": "",
                    }
                ],
            },
        )

        a = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux", Workflow.LINUX, unit_tests=False)
        self.assertTrue(a.merge(b), "Should merge jobs that have different unit test configurations.")
        self.assertEqual(a, JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True))

        a = handle_preset("linux-unit-tests")
        a = handle_modifier(a, "linux-unit-tests")
        b = handle_preset("linux-wpt")
        b = handle_modifier(b, "linux-wpt")
        assert a is not None
        assert b is not None
        self.assertTrue(a.merge(b), "Should merge jobs that have different unit test configurations.")
        self.assertEqual(a, JobConfig("Linux (Unit Tests, WPT)", Workflow.LINUX, unit_tests=True, wpt=True))

        a = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Mac", Workflow.MACOS, unit_tests=True)
        self.assertFalse(a.merge(b), "Should not merge jobs with different workflows.")
        self.assertEqual(a, JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True))

        a = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux (Unit Tests, Production)", Workflow.LINUX, unit_tests=True, profile="production")
        self.assertFalse(a.merge(b), "Should not merge jobs with different profiles.")
        self.assertEqual(a, JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True))

        a = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True, wpt_args="/css")
        self.assertFalse(a.merge(b), "Should not merge jobs that run different WPT tests.")
        self.assertEqual(a, JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True))

        a = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True)
        b = JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True, build_args="--help")
        self.assertFalse(a.merge(b), "Should not merge jobs with different build arguments.")
        self.assertEqual(a, JobConfig("Linux (Unit Tests)", Workflow.LINUX, unit_tests=True))

    def test_full(self) -> None:
        self.assertDictEqual(json.loads(Config("full").to_json()), json.loads(Config("").to_json()))

    def test_wpt_alias(self) -> None:
        self.assertDictEqual(json.loads(Config("wpt").to_json()), json.loads(Config("linux-wpt").to_json()))


def run_tests() -> bool:
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    suite = unittest.TestLoader().loadTestsFromTestCase(TestParser)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
