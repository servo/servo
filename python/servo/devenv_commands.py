# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.
import argparse
import json
import os
import shlex
from subprocess import CompletedProcess
from typing import Any

from mach.decorators import (
    Command,
    CommandArgument,
    CommandProvider,
)
from tidy.linting_report import GitHubAnnotationManager

from servo.command_base import CommandBase, call


@CommandProvider
class MachCommands(CommandBase):
    @Command(
        "print-env",
        description="Print (POSIX) shell export statements for environment variables set by mach when building servo",
        category="devenv",
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def print_env(self, **_kwargs: Any) -> int:
        env = self.build_env()
        for key in sorted(env):
            value = env[key]
            # The build_env command also reads environment variables from the env it copies,
            # so we can't build a "clean" env and need to filter here instead.
            if os.environ.get(key) != value:
                print(f"export {key}={shlex.quote(value)}")
        return 0

    @Command(
        "exec",
        description="Execute a command in the build environment",
        category="devenv",
    )
    @CommandArgument(
        "command",
        nargs=argparse.REMAINDER,
        help="Command to execute in the build environment. \
            Use `--` to delimit the start of the command, \
            if arguments to the command are falsely interpreted as mach arguments",
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def exec_command(self, command: list[str], **_kwargs: Any) -> int:
        if not command:
            print("No command provided. Pass a command to execute.")
            return 1

        self.ensure_bootstrapped()
        return call(command, env=self.build_env())

    @Command("check", description='Run "cargo check"', category="devenv")
    @CommandArgument(
        "params", default=None, nargs="...", help="Command-line arguments to be passed through to cargo check"
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def check(self, params: list[str], **kwargs: Any) -> int:
        if not params:
            params = []

        self.ensure_bootstrapped()
        status = self.run_cargo_build_like_command("check", params, **kwargs)
        assert isinstance(status, int)
        if status == 0:
            print("Finished checking, binary NOT updated. Consider ./mach build before ./mach run")

        return status

    @Command("rustc", description="Run the Rust compiler", category="devenv")
    @CommandArgument("params", default=None, nargs="...", help="Command-line arguments to be passed through to rustc")
    def rustc(self, params: list[str]) -> int:
        if params is None:
            params = []

        self.ensure_bootstrapped()
        return call(["rustc"] + params, env=self.build_env())

    @Command("cargo-fix", description='Run "cargo fix"', category="devenv")
    @CommandArgument(
        "params", default=None, nargs="...", help="Command-line arguments to be passed through to cargo-fix"
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def cargo_fix(self, params: list[str], **kwargs: Any) -> int:
        if not params:
            params = []

        self.ensure_bootstrapped()
        status = self.run_cargo_build_like_command("fix", params, **kwargs)
        assert isinstance(status, int)
        return status

    @Command("clippy", description='Run "cargo clippy.', category="devenv")
    @CommandArgument(
        "params",
        default=None,
        nargs="...",
        help="Command-line arguments to be passed through to clippy. "
        "Note that this can be separated via `--` from arguments for `mach`. "
        "Arguments for clippy itself need another `--`, e.g. `./mach clippy -- -- --deny clippy::lint_name",
    )
    @CommandArgument(
        "--github-annotations",
        default=False,
        action="store_true",
        help="Emit the clippy warnings in the Github Actions annotations format",
    )
    @CommandBase.common_command_arguments(build_configuration=True, build_type=False)
    def cargo_clippy(self, params: list[str], github_annotations: bool = False, **kwargs: Any) -> int:
        if not params:
            params = []

        if kwargs.get("use_crown", False):
            print(
                "Error: `clippy` and `--use-crown` cannot be used together.\n"
                "`clippy` takes precedence over `crown`, so `crown` would not run.\n"
                "Please use `./mach check --use-crown` instead to run the crown linter."
            )
            return 1

        self.ensure_bootstrapped()
        env = self.build_env()
        env["RUSTC"] = "rustc"

        # arguments to be passed through to clippy (as opposed to the cargo clippy wrapper)
        # These should mainly be `--allow`, `--warn`, `--deny`, `--forbid`
        # Note that some lints can additionally be configured by `.clippy.toml` at the repository root.
        clippy_args = ["--deny=clippy::disallowed_types", "--warn=clippy::redundant-clone"]

        if "--" not in params:
            params.append("--")
        params.extend(clippy_args)

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
