# Copyright 2024 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from typing import List, Tuple, TypeGuard
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import toml
from enum import Enum

from os import path
from packaging.version import parse as parse_version
from typing import Any, Optional

import servo.platform
import servo.util as util


class SanitizerKind(Enum):
    NONE = 0
    ASAN = 1
    TSAN = 2

    # Apparently enums don't always compare across modules, so we define
    # helper methods.
    def is_asan(self) -> bool:
        return self is self.ASAN

    def is_tsan(self) -> bool:
        return self is self.TSAN

    # Returns true if no sanitizer is enabled.
    def is_none(self) -> bool:
        return self is self.NONE

    # Returns true if a sanitizer is enabled.
    def is_some(self) -> bool:
        return not self.is_none()


class BuildTarget(object):
    def __init__(self, target_triple: str) -> None:
        self.target_triple = target_triple

    @staticmethod
    def from_triple(target_triple: Optional[str]) -> "BuildTarget":
        host_triple = servo.platform.host_triple()
        if target_triple:
            if "android" in target_triple:
                return AndroidTarget(target_triple)
            elif "ohos" in target_triple:
                return OpenHarmonyTarget(target_triple)
            elif target_triple != host_triple:
                raise Exception(f"Unknown build target {target_triple}")
        return BuildTarget(host_triple)

    def triple(self) -> str:
        return self.target_triple

    def binary_name(self) -> str:
        return f"servoshell{servo.platform.get().executable_suffix()}"

    def configure_build_environment(self, env: dict[str, str], config: dict[str, Any], topdir: pathlib.Path) -> None:
        pass

    def is_cross_build(self) -> bool:
        return False

    def needs_packaging(self) -> bool:
        return False


class CrossBuildTarget(BuildTarget):
    def is_cross_build(self) -> bool:
        return True


class AndroidTarget(CrossBuildTarget):
    DEFAULT_TRIPLE = "aarch64-linux-android"

    @staticmethod
    def min_sdk() -> int:
        """Minimum supported Android API level"""
        version_catalog_file = path.join(util.SERVO_ROOT, "support", "android", "apk", "gradle", "libs.versions.toml")
        return int(toml.load(version_catalog_file)["versions"]["android-sdk-min"])

    def ndk_configuration(self) -> dict[str, str]:
        target = self.triple()
        api = self.min_sdk()
        config = {}
        if target == "armv7-linux-androideabi":
            config["platform"] = f"android-{api}"
            config["target"] = target
            config["toolchain_prefix"] = "arm-linux-androideabi"
            config["arch"] = "arm"
            config["lib"] = "armeabi-v7a"
            config["toolchain_name"] = f"armv7a-linux-androideabi{api}"
        elif target == "aarch64-linux-android":
            config["platform"] = f"android-{api}"
            config["target"] = target
            config["toolchain_prefix"] = target
            config["arch"] = "arm64"
            config["lib"] = "arm64-v8a"
            config["toolchain_name"] = f"aarch64-linux-androideabi{api}"
        elif target == "i686-linux-android":
            # https://github.com/jemalloc/jemalloc/issues/1279
            config["platform"] = f"android-{api}"
            config["target"] = target
            config["toolchain_prefix"] = target
            config["arch"] = "x86"
            config["lib"] = "x86"
            config["toolchain_name"] = f"i686-linux-android{api}"
        elif target == "x86_64-linux-android":
            config["platform"] = f"android-{api}"
            config["target"] = target
            config["toolchain_prefix"] = target
            config["arch"] = "x86_64"
            config["lib"] = "x86_64"
            config["toolchain_name"] = f"x86_64-linux-android{api}"
        else:
            raise Exception(f"Unknown android target {target}")

        return config

    def configure_build_environment(self, env: dict[str, str], config: dict[str, Any], topdir: pathlib.Path) -> None:
        # Paths to Android build tools:
        if config["android"]["sdk"]:
            env["ANDROID_SDK_ROOT"] = config["android"]["sdk"]
        if config["android"]["ndk"]:
            env["ANDROID_NDK_ROOT"] = config["android"]["ndk"]

        if "IN_NIX_SHELL" in env and ("ANDROID_NDK_ROOT" not in env or "ANDROID_SDK_ROOT" not in env):
            print("Please set SERVO_ANDROID_BUILD=1 when starting the Nix shell to include the Android SDK/NDK.")
            sys.exit(1)
        if "ANDROID_NDK_ROOT" not in env:
            print("Please set the ANDROID_NDK_ROOT environment variable.")
            sys.exit(1)
        if "ANDROID_SDK_ROOT" not in env:
            print("Please set the ANDROID_SDK_ROOT environment variable.")
            sys.exit(1)

        ndk_configuration = self.ndk_configuration()
        android_platform = ndk_configuration["platform"]
        android_toolchain_name = ndk_configuration["toolchain_name"]
        android_api = android_platform.replace("android-", "")

        # Check if the NDK version is 28
        if not os.path.isfile(path.join(env["ANDROID_NDK_ROOT"], "source.properties")):
            print("ANDROID_NDK should have file `source.properties`.")
            print("The environment variable ANDROID_NDK_ROOT may be set at a wrong path.")
            sys.exit(1)
        with open(path.join(env["ANDROID_NDK_ROOT"], "source.properties"), encoding="utf8") as ndk_properties:
            lines = ndk_properties.readlines()
            if lines[1].split(" = ")[1].split(".")[0] != "28":
                print("Servo currently only supports NDK r28.")
                sys.exit(1)

        # Android builds also require having the gcc bits on the PATH and various INCLUDE
        # path munging if you do not want to install a standalone NDK. See:
        # https://dxr.mozilla.org/mozilla-central/source/build/autoconf/android.m4#139-161
        os_type = platform.system().lower()
        if os_type not in ["linux", "darwin"]:
            raise Exception("Android cross builds are only supported on Linux and macOS.")

        llvm_prebuilt = path.join(env["ANDROID_NDK_ROOT"], "toolchains", "llvm", "prebuilt")

        cpu_type = platform.machine().lower()
        host_suffix = "unknown"
        if cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
            host_suffix = "x86"
        elif cpu_type in ["x86_64", "x86-64", "x64", "amd64"]:
            host_suffix = "x86_64"
        else:
            available_prebuilts = os.listdir(llvm_prebuilt)
            available_prebuilts = [prebuilt for prebuilt in available_prebuilts if prebuilt.startswith(os_type)]
            # If there is only one prebuilt option available, it's probably the right one for the host
            # platform. E.g. on Arm macs, only the x86 prebuilts are available, buts that perfectly fine,
            # since there is rosetta.
            if len(available_prebuilts) == 1:
                host_suffix = available_prebuilts[0].removeprefix(f"{os_type}-")
            else:
                print(f"Error: Can't determine LLVM prebuilt. Unknown cpu type {cpu_type}.")
                print(f"Hint: The LLVM prebuilts folder contains the following entries: {available_prebuilts}")
                print("Please open an issue with the above information")
                raise Exception("Can't determine LLVM prebuilt directory.")
        host = os_type + "-" + host_suffix

        host_cc = env.get("HOST_CC") or shutil.which("clang")
        host_cxx = env.get("HOST_CXX") or shutil.which("clang++")

        llvm_toolchain = path.join(llvm_prebuilt, host)
        env["PATH"] = env["PATH"] + ":" + path.join(llvm_toolchain, "bin")

        def to_ndk_bin(prog: str) -> str:
            return path.join(llvm_toolchain, "bin", prog)

        # This workaround is due to an issue in the x86_64 Android NDK that introduces
        # an undefined reference to the symbol '__extendsftf2'.
        # See https://github.com/termux/termux-packages/issues/8029#issuecomment-1369150244
        if "x86_64" in self.triple():
            libclangrt_filename = subprocess.run(
                [to_ndk_bin(f"x86_64-linux-android{android_api}-clang"), "--print-libgcc-file-name"],
                check=True,
                capture_output=True,
                encoding="utf8",
            ).stdout
            env["RUSTFLAGS"] = env.get("RUSTFLAGS", "")
            env["RUSTFLAGS"] += f" -C link-arg={libclangrt_filename}"

        assert host_cc
        assert host_cxx

        env["RUST_TARGET"] = self.triple()
        env["HOST_CC"] = host_cc
        env["HOST_CXX"] = host_cxx
        env["HOST_CFLAGS"] = ""
        env["HOST_CXXFLAGS"] = ""
        env["TARGET_CC"] = to_ndk_bin("clang")
        env["TARGET_CPP"] = to_ndk_bin("clang") + " -E"
        env["TARGET_CXX"] = to_ndk_bin("clang++")

        env["TARGET_AR"] = to_ndk_bin("llvm-ar")
        env["TARGET_RANLIB"] = to_ndk_bin("llvm-ranlib")
        env["TARGET_OBJCOPY"] = to_ndk_bin("llvm-objcopy")
        env["TARGET_YASM"] = to_ndk_bin("yasm")
        env["TARGET_STRIP"] = to_ndk_bin("llvm-strip")
        env["RUST_FONTCONFIG_DLOPEN"] = "on"

        # Needed for tikv-jemalloc, which doesn't respect TARGET_AR and co.
        # On macos this lead to it falling back to `ar` and missing jemalloc
        # symbols in libservoshell.so.
        env["AR"] = to_ndk_bin("llvm-ar")
        env["RANLIB"] = to_ndk_bin("llvm-ranlib")

        env["LIBCLANG_PATH"] = path.join(llvm_toolchain, "lib")
        env["CLANG_PATH"] = to_ndk_bin("clang")
        env["BINDGEN_EXTRA_CLANG_ARGS"] = (
            f"--target={android_toolchain_name} --sysroot={path.join(llvm_toolchain, 'sysroot')}"
        )

        # A cheat-sheet for some of the build errors caused by getting the search path wrong...
        #
        # fatal error: 'limits' file not found
        #   -- add -I cxx_include
        # unknown type name '__locale_t' (when running bindgen in mozjs_sys)
        #   -- add -isystem sysroot_include
        # error: use of undeclared identifier 'UINTMAX_C'
        #   -- add -D__STDC_CONSTANT_MACROS
        #
        # Also worth remembering: autoconf uses C for its configuration,
        # even for C++ builds, so the C flags need to line up with the C++ flags.
        env["TARGET_CFLAGS"] = env.get("TARGET_CFLAGS", "") + " " + "--target=" + android_toolchain_name
        env["TARGET_CXXFLAGS"] = env.get("TARGET_CXXFLAGS", "") + " " + "--target=" + android_toolchain_name

        # These two variables are needed for the mozjs compilation.
        env["ANDROID_API_LEVEL"] = android_api
        env["ANDROID_NDK_HOME"] = env["ANDROID_NDK_ROOT"]
        env["TARGET_PKG_CONFIG_SYSROOT_DIR"] = path.join(llvm_toolchain, "sysroot")

    def binary_name(self) -> str:
        return "libservoshell.so"

    def is_cross_build(self) -> bool:
        return True

    def needs_packaging(self) -> bool:
        return True

    def get_package_path(self, build_type_directory: str) -> str:
        base_path = util.get_target_dir()
        base_path = path.join(base_path, self.triple())
        apk_name = "servoapp.apk"
        return path.join(base_path, build_type_directory, apk_name)


class OpenHarmonyTarget(CrossBuildTarget):
    DEFAULT_TRIPLE = "aarch64-unknown-linux-ohos"
    # The minimum SDK level we support.
    MINIMUM_OHOS_API_LEVEL = 20
    # The layout of the dict might change in the future, and backwords incompatible changes
    # will bump the schema version in cargo-ohos.
    CARGO_OHOS_EXPECTED_SCHEMA_VERSION = 1
    # Pin a cargo-ohos semver version for bootstrap
    REQUESTED_CARGO_OHOS_VERSION = "0.3"

    @classmethod
    def is_cargo_ohos_compatible(cls) -> Tuple[bool, Optional[str]]:
        """Returns true if the cargo-ohos version is compatible. False if we need to reinstall.
        When false, the second return parameter is a string with error information.
        """

        def cargo_ohos_version() -> Optional[str]:
            if shutil.which("cargo-ohos") is None:
                return None
            result = subprocess.run(["cargo-ohos", "ohos", "--version"], encoding="utf-8", capture_output=True)
            if result.returncode != 0:
                return None
            version = result.stdout.strip().split(" ")[-1]
            return version

        found_version = cargo_ohos_version()
        if found_version is None:
            return (False, "cargo-ohos not installed")
        found_version = parse_version(found_version)
        required_version = parse_version(cls.REQUESTED_CARGO_OHOS_VERSION)
        semver_incompatible_error_msg = (
            f"The installed cargo-ohos version {found_version} is not SemVer compatible"
            + f" with the required cargo-ohos version {required_version}"
        )
        if required_version > found_version:
            return (False, f"The installed cargo-ohos version {found_version} is too old.")
        if required_version.major == 0:
            if required_version.minor != found_version.minor:
                return (False, semver_incompatible_error_msg)
        else:
            if required_version.major != found_version.major:
                return (False, semver_incompatible_error_msg)
        return (True, None)

    def get_cargo_ohos_env(self, input_env: dict[str, str]) -> dict[str, Any]:
        (compatible, error_msg) = self.is_cargo_ohos_compatible()
        if not compatible:
            print(f"Building for OpenHarmony requires `cargo-ohos`: {error_msg}", file=sys.stderr)
            print("Please rerun `./mach bootstrap --ohos`.", file=sys.stderr)
            sys.exit(1)
        command = ["cargo", "ohos", "env", "--format", "json", "--target", self.triple()]
        try:
            output = subprocess.run(command, check=True, capture_output=True, encoding="utf8", env=input_env).stdout
        except subprocess.CalledProcessError as exception:
            print(exception.stderr, end="", file=sys.stderr)
            print("Failed to determine the OpenHarmony toolchain environment via `cargo ohos env`.", file=sys.stderr)
            sys.exit(1)
        try:
            ohos_env = json.loads(output)
        except json.JSONDecodeError as error:
            print(f"Failed to parse `cargo ohos env` output as JSON: {error}", file=sys.stderr)
            sys.exit(1)
        schema_version = ohos_env.get("schema_version")
        if schema_version is None or schema_version != self.CARGO_OHOS_EXPECTED_SCHEMA_VERSION:
            # This shouldn't happen if cargo-ohos releases follow semver, but still better
            # to check.
            raise RuntimeError("Unexpected schema-version mismatch.")

        return ohos_env

    def configure_build_environment(self, env: dict[str, str], config: dict[str, Any], topdir: pathlib.Path) -> None:
        # Paths to OpenHarmony SDK and build tools:
        # Note: `OHOS_SDK_NATIVE` is the CMake variable name the `hvigor` build-system
        # uses for the native directory of the SDK, so we use the same name to be consistent.
        if "OHOS_SDK_NATIVE" not in env and config["ohos"]["ndk"]:
            env["OHOS_SDK_NATIVE"] = config["ohos"]["ndk"]

        ohos_info = self.get_cargo_ohos_env(env)
        ohos_env = ohos_info["env"]

        sdk_info: dict[str, str] = ohos_info["sdk"]
        ohos_api_version: Optional[int] = None
        try:
            ohos_api_version = int(sdk_info["api_version"])
        except TypeError:
            print("Error: cargo-ohos was unable to determine the SDK API version", file=sys.stderr)
            sys.exit(1)

        ohos_sdk_version = sdk_info["version"]

        print(
            f"Info: The OpenHarmony SDK {ohos_sdk_version} is targeting API-level {ohos_api_version}",
            file=sys.stderr,
        )

        if ohos_api_version < self.MINIMUM_OHOS_API_LEVEL:
            print(
                "Error: Building servo for OpenHarmony requires an SDK version with"
                f"API level {self.MINIMUM_OHOS_API_LEVEL} or newer",
                file=sys.stderr,
            )
            sys.exit(1)

        # Cargo prefers `CARGO_ENCODED_RUSTFLAGS` if set, but mach currently uses `RUSTFLAGS`
        # instead, so we remove this from the environment. It probably would make sense to migrate
        # mach towards also using the encoded form.
        del ohos_env["CARGO_ENCODED_RUSTFLAGS"]

        env.update(ohos_env)
        # `cargo ohos` currently doesn't set RUSTFLAGS in the environment (since it uses the encoded rustflags),
        # So we explicitly add the flags here and preserve any existing rustflags.
        env["RUSTFLAGS"] += " " + " ".join(ohos_info["flags"]["rustflags"])

        # CC and CXX should already be set to appropriate host compilers by `build_env()`
        # TODO: HOST_CC and HOST_CXX are needed for mozjs, and should be set in the mozjs
        # buildscript in the future (also for android).
        env["HOST_CC"] = env["CC"]
        env["HOST_CXX"] = env["CXX"]
        env["HOST_CFLAGS"] = ""
        env["HOST_CXXFLAGS"] = ""

        sanitizer: SanitizerKind = config["build"]["sanitizer"]
        san_compile_flags = []
        if sanitizer.is_some():
            # TODO: Probably this could also be done by cargo-ohos in the future
            san_compile_flags: List[str] = []
            link_args: List[str] = []
            clang_target_triple = ohos_info["target"]["clang_triple"]
            # Lookup `<sdk>/native/llvm/lib/clang/15.0.4/lib/aarch64-linux-ohos/libclang_rt.asan.so`
            lib_clang = pathlib.Path(ohos_info["toolchain"]["libclang_dir"], "clang")
            children = [f.path for f in os.scandir(lib_clang) if f.is_dir()]
            if len(children) != 1:
                raise RuntimeError(f"Expected exactly 1 libclang version: `{children}`")
            lib_clang_version_dir = pathlib.Path(children[0])
            libclang_arch = lib_clang_version_dir.joinpath("lib", clang_target_triple).resolve()
            # Use the clangrt from the NDK to use the same library for both C++ and Rust.
            env["RUSTFLAGS"] += " -Zexternal-clangrt"
            san_compile_flags.append("-fno-omit-frame-pointer")

            # On OpenHarmony we add some additional flags when asan is enabled
            if sanitizer.is_asan():
                libasan_so_path = libclang_arch.joinpath("libclang_rt.asan.so")
                libasan_preinit_path = libclang_arch.joinpath("libclang_rt.asan-preinit.a")
                if not libasan_so_path.exists():
                    raise RuntimeError(f"Couldn't find ASAN runtime library at {libasan_so_path}")
                link_args.extend(
                    [
                        "-fsanitize=address",
                        "--rtlib=compiler-rt",
                        "-shared-libasan",
                        str(libasan_so_path),
                        "-Wl,--whole-archive",
                        "-Wl," + str(libasan_preinit_path),
                        "-Wl,--no-whole-archive",
                    ]
                )

                san_compile_flags.extend(["-fsanitize=address", "-shared-libasan", "-fsanitize-recover=address"])

                arch_asan_ignore_list = lib_clang_version_dir.joinpath("share", "asan_ignorelist.txt")
                if arch_asan_ignore_list.exists():
                    san_compile_flags.append("-fsanitize-system-ignorelist=" + str(arch_asan_ignore_list))
                else:
                    print(f"Warning: Couldn't find system ASAN ignorelist at `{arch_asan_ignore_list}`")
            elif sanitizer.is_tsan():
                libtsan_so_path = libclang_arch.joinpath("libclang_rt.tsan.so")
                builtins_path = libclang_arch.joinpath("libclang_rt.builtins.a")

                link_args.extend(
                    [
                        "-fsanitize=thread",
                        "--rtlib=compiler-rt",
                        "-shared-libsan",
                        str(libtsan_so_path),
                        str(builtins_path),
                    ]
                )
                san_compile_flags.append("-shared-libsan")

            link_args = [f"-Clink-arg={arg}" for arg in link_args]
            env["RUSTFLAGS"] += " " + " ".join(link_args)
            # If a non-SDK LLVM is used with cargo-ohos, then these flags will not be set.
            env["TARGET_CFLAGS"] = env.get("TARGET_CFLAGS", "") + " " + " ".join(san_compile_flags)
            env["TARGET_CXXFLAGS"] = env.get("TARGET_CXXFLAGS", "") + " " + " ".join(san_compile_flags)

    def binary_name(self) -> str:
        return "libservoshell.so"

    def needs_packaging(self) -> bool:
        return True

    def get_package_path(self, build_type_directory: str, flavor: Optional[str] = None) -> str:
        base_path = util.get_target_dir()
        base_path = path.join(base_path, "openharmony", self.triple())
        hap_name = "servoshell-default-signed.hap"
        if not flavor:
            flavor = "default"
        build_output_path = path.join("entry", "build", flavor, "outputs", "default")
        return path.join(base_path, build_type_directory, build_output_path, hap_name)

    def abi_string(self) -> str:
        abi_map = {"aarch64-unknown-linux-ohos": "arm64-v8a", "x86_64-unknown-linux-ohos": "x86_64"}
        return abi_map[self.triple()]


def is_android(target: BuildTarget) -> TypeGuard[AndroidTarget]:
    return isinstance(target, AndroidTarget)


def is_openharmony(target: BuildTarget) -> TypeGuard[OpenHarmonyTarget]:
    return isinstance(target, OpenHarmonyTarget)
