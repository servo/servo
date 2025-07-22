# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from subprocess import CompletedProcess
import json

from mach.decorators import (
    Command,
    CommandArgument,
    CommandProvider,
)
from tidy.linting_report import GitHubAnnotationManager

from servo.command_base import CommandBase, call


@CommandProvider
class MachCommands(CommandBase):
    @Command("check", description='Run "cargo check"', category="devenv")
    @CommandArgument(
        "params", default=None, nargs="...", help="Command-line arguments to be passed through to cargo check"
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def check(self, params, **kwargs) -> int:
        if not params:
            params = []

        self.ensure_bootstrapped()
        self.ensure_clobbered()
        status = self.run_cargo_build_like_command("check", params, **kwargs)
        assert isinstance(status, int)
        if status == 0:
            print("Finished checking, binary NOT updated. Consider ./mach build before ./mach run")

        return status

    @Command("rustc", description="Run the Rust compiler", category="devenv")
    @CommandArgument("params", default=None, nargs="...", help="Command-line arguments to be passed through to rustc")
    def rustc(self, params) -> int:
        if params is None:
            params = []

        self.ensure_bootstrapped()
        return call(["rustc"] + params, env=self.build_env())

    @Command("cargo-fix", description='Run "cargo fix"', category="devenv")
    @CommandArgument(
        "params", default=None, nargs="...", help="Command-line arguments to be passed through to cargo-fix"
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def cargo_fix(self, params, **kwargs) -> int:
        if not params:
            params = []

        self.ensure_bootstrapped()
        self.ensure_clobbered()
        status = self.run_cargo_build_like_command("fix", params, **kwargs)
        assert isinstance(status, int)
        return status

    @Command("clippy", description='Run "cargo clippy"', category="devenv")
    @CommandArgument("params", default=None, nargs="...", help="Command-line arguments to be passed through to clippy")
    @CommandArgument(
        "--github-annotations",
        default=False,
        action="store_true",
        help="Emit the clippy warnings in the Github Actions annotations format",
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def cargo_clippy(self, params, github_annotations=False, **kwargs) -> int:
        if not params:
            params = []

        self.ensure_bootstrapped()
        self.ensure_clobbered()
        env = self.build_env()
        env["RUSTC"] = "rustc"

        if github_annotations:
            if "--message-format=json" not in params:
                params.insert(0, "--message-format=json")

            github_annotation_manager = GitHubAnnotationManager("clippy")

            results = self.run_cargo_build_like_command("clippy", params, env=env, capture_output=True, **kwargs)
            assert isinstance(results, CompletedProcess)
            if results.returncode == 0:
                return 0
            try:
                github_annotation_manager.emit_annotations_for_clippy(
                    [json.loads(line) for line in results.stdout.splitlines() if line.strip()]
                )
            except json.JSONDecodeError:
                pass
            return results.returncode
        status = self.run_cargo_build_like_command("clippy", params, env=env, **kwargs)
        assert isinstance(status, int)
        return status

    @Command("fetch", description="Fetch Rust, Cargo and Cargo dependencies", category="devenv")
    def fetch(self) -> int:
        self.ensure_bootstrapped()
        return call(["cargo", "fetch"], env=self.build_env())
