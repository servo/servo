# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest
from python.servo.build_commands import CommandBase


class DummyContext:
    def __init__(self) -> None:
        self.topdir = ""
        self.verbose = False


class TestCommandBase(unittest.TestCase):
    def setUp(self) -> None:
        self.ctx = DummyContext()
        self.cmd = CommandBase(self.ctx)

    def test_clang_version_ubuntu(self) -> None:
        output = (
            "Ubuntu clang version 18.1.3 (1ubuntu1)\n"
            "Target: x86_64-pc-linux-gnu\n"
            "Thread model: posix\n"
            "InstalledDir: /usr/bin\n"
        )
        self.assertEqual(
            self.cmd.parse_major_version_from_stdout(output),
            "18",
        )

    def test_clang_version_macos(self) -> None:
        output = (
            "Apple clang version 17.0.0 (clang-1700.6.3.2)\n"
            "Target: arm64-apple-darwin25.2.0\n"
            "Thread model: posix\n"
            "InstalledDir: /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin\\n"
        )

        self.assertEqual(
            self.cmd.parse_major_version_from_stdout(output),
            "17",
        )

    def test_clang_version_fedora(self) -> None:
        output = (
            "clang version 21.1.8 (Fedora 21.1.8-4.fc43)\n"
            "Target: x86_64-redhat-linux-gnu\n"
            "Thread model: posix\n"
            "InstalledDir: /usr/bin\n"
            "Configuration file: /etc/clang/x86_64-redhat-linux-gnu-clang.cfg\n"
        )

        self.assertEqual(
            self.cmd.parse_major_version_from_stdout(output),
            "21",
        )


def run_tests() -> bool:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(TestCommandBase)
    result = unittest.TextTestRunner().run(suite)
    return result.wasSuccessful()
