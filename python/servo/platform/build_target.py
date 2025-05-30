# Copyright 2024 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import errno
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
from enum import Enum

from os import path
from packaging.version import parse as parse_version
from typing import Any, Dict, Optional

import servo.platform
import servo.util as util


class SanitizerKind(Enum):
    UNKNOWN = 0
    ASAN = 1


class BuildTarget(object):
    def __init__(self, target_triple: str):
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
        return f"servo{servo.platform.get().executable_suffix()}"

    def configure_build_environment(self, env: Dict[str, str], config: Dict[str, Any], topdir: pathlib.Path):
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

    def ndk_configuration(self) -> Dict[str, str]:
        target = self.triple()
        config = {}
        if target == "armv7-linux-androideabi":
            config["platform"] = "android-30"
            config["target"] = target
            config["toolchain_prefix"] = "arm-linux-androideabi"
            config["arch"] = "arm"
            config["lib"] = "armeabi-v7a"
            config["toolchain_name"] = "armv7a-linux-androideabi30"
        elif target == "aarch64-linux-android":
            config["platform"] = "android-30"
            config["target"] = target
            config["toolchain_prefix"] = target
            config["arch"] = "arm64"
            config["lib"] = "arm64-v8a"
            config["toolchain_name"] = "aarch64-linux-androideabi30"
        elif target == "i686-linux-android":
            # https://github.com/jemalloc/jemalloc/issues/1279
            config["platform"] = "android-30"
            config["target"] = target
            config["toolchain_prefix"] = target
            config["arch"] = "x86"
            config["lib"] = "x86"
            config["toolchain_name"] = "i686-linux-android30"
        elif target == "x86_64-linux-android":
            config["platform"] = "android-30"
            config["target"] = target
            config["toolchain_prefix"] = target
            config["arch"] = "x86_64"
            config["lib"] = "x86_64"
            config["toolchain_name"] = "x86_64-linux-android30"
        else:
            raise Exception(f"Unknown android target {target}")

        return config

    def configure_build_environment(self, env: Dict[str, str], config: Dict[str, Any], topdir: pathlib.Path):
        # Paths to Android build tools:
        if config["android"]["sdk"]:
            env["ANDROID_SDK_ROOT"] = config["android"]["sdk"]
        if config["android"]["ndk"]:
            env["ANDROID_NDK_ROOT"] = config["android"]["ndk"]

        toolchains = path.join(topdir, "android-toolchains")
        for kind in ["sdk", "ndk"]:
            default = os.path.join(toolchains, kind)
            if os.path.isdir(default):
                env.setdefault(f"ANDROID_{kind.upper()}_ROOT", default)

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
        android_lib = ndk_configuration["lib"]

        android_api = android_platform.replace("android-", "")

        # Check if the NDK version is 26
        if not os.path.isfile(path.join(env["ANDROID_NDK_ROOT"], "source.properties")):
            print("ANDROID_NDK should have file `source.properties`.")
            print("The environment variable ANDROID_NDK_ROOT may be set at a wrong path.")
            sys.exit(1)
        with open(path.join(env["ANDROID_NDK_ROOT"], "source.properties"), encoding="utf8") as ndk_properties:
            lines = ndk_properties.readlines()
            if lines[1].split(" = ")[1].split(".")[0] != "26":
                print("Servo currently only supports NDK r26c.")
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

        def to_ndk_bin(prog):
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
            env["RUSTFLAGS"] += f"-C link-arg={libclangrt_filename}"

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

        env["LIBCLANG_PATH"] = path.join(llvm_toolchain, "lib")
        env["CLANG_PATH"] = to_ndk_bin("clang")

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
        env["TARGET_CFLAGS"] = "--target=" + android_toolchain_name
        env["TARGET_CXXFLAGS"] = "--target=" + android_toolchain_name

        # These two variables are needed for the mozjs compilation.
        env["ANDROID_API_LEVEL"] = android_api
        env["ANDROID_NDK_HOME"] = env["ANDROID_NDK_ROOT"]

        # The two variables set below are passed by our custom
        # support/android/toolchain.cmake to the NDK's CMake toolchain file
        env["ANDROID_ABI"] = android_lib
        env["ANDROID_PLATFORM"] = android_platform
        env["NDK_CMAKE_TOOLCHAIN_FILE"] = path.join(
            env["ANDROID_NDK_ROOT"], "build", "cmake", "android.toolchain.cmake"
        )
        env["CMAKE_TOOLCHAIN_FILE"] = path.join(topdir, "support", "android", "toolchain.cmake")

        # Set output dir for gradle aar files
        env["AAR_OUT_DIR"] = path.join(topdir, "target", "android", "aar")
        if not os.path.exists(env["AAR_OUT_DIR"]):
            os.makedirs(env["AAR_OUT_DIR"])

        env["TARGET_PKG_CONFIG_SYSROOT_DIR"] = path.join(llvm_toolchain, "sysroot")

    def binary_name(self) -> str:
        return "libservoshell.so"

    def is_cross_build(self) -> bool:
        return True

    def needs_packaging(self) -> bool:
        return True

    def get_package_path(self, build_type_directory: str) -> str:
        base_path = util.get_target_dir()
        base_path = path.join(base_path, "android", self.triple())
        apk_name = "servoapp.apk"
        return path.join(base_path, build_type_directory, apk_name)


class OpenHarmonyTarget(CrossBuildTarget):
    DEFAULT_TRIPLE = "aarch64-unknown-linux-ohos"

    def configure_build_environment(self, env: Dict[str, str], config: Dict[str, Any], topdir: pathlib.Path):
        # Paths to OpenHarmony SDK and build tools:
        # Note: `OHOS_SDK_NATIVE` is the CMake variable name the `hvigor` build-system
        # uses for the native directory of the SDK, so we use the same name to be consistent.
        if "OHOS_SDK_NATIVE" not in env and config["ohos"]["ndk"]:
            env["OHOS_SDK_NATIVE"] = config["ohos"]["ndk"]

        if "OHOS_SDK_NATIVE" not in env:
            print(
                "Please set the OHOS_SDK_NATIVE environment variable to the location of the `native` directory "
                "in the OpenHarmony SDK."
            )
            sys.exit(1)

        ndk_root = pathlib.Path(env["OHOS_SDK_NATIVE"])

        if not ndk_root.is_dir():
            print(f"OHOS_SDK_NATIVE is not set to a valid directory: `{ndk_root}`")
            sys.exit(1)

        ndk_root = ndk_root.resolve()
        package_info = ndk_root.joinpath("oh-uni-package.json")
        try:
            with open(package_info) as meta_file:
                meta = json.load(meta_file)
            ohos_api_version = int(meta["apiVersion"])
            ohos_sdk_version = parse_version(meta["version"])
            if ohos_sdk_version < parse_version("5.0") or ohos_api_version < 12:
                raise RuntimeError("Building servo for OpenHarmony requires SDK version 5.0 (API-12) or newer.")
            print(f"Info: The OpenHarmony SDK {ohos_sdk_version} is targeting API-level {ohos_api_version}")
        except (OSError, json.JSONDecodeError) as e:
            print(f"Failed to read metadata information from {package_info}")
            print(f"Exception: {e}")

        llvm_toolchain = ndk_root.joinpath("llvm")
        llvm_bin = llvm_toolchain.joinpath("bin")
        ohos_sysroot = ndk_root.joinpath("sysroot")
        if not (llvm_toolchain.is_dir() and llvm_bin.is_dir()):
            print(f"Expected to find `llvm` and `llvm/bin` folder under $OHOS_SDK_NATIVE at `{llvm_toolchain}`")
            sys.exit(1)
        if not ohos_sysroot.is_dir():
            print(f"Could not find OpenHarmony sysroot in {ndk_root}")
            sys.exit(1)
        # When passing the sysroot to Rust crates such as `cc-rs` or bindgen, we should pass
        # POSIX paths, since otherwise the backslashes in windows paths may be interpreted as
        # escapes and lead to errors.
        ohos_sysroot_posix = ohos_sysroot.as_posix()

        # Note: We don't use the `<target_triple>-clang` wrappers on purpose, since
        # a) the OH 4.0 SDK does not have them yet AND
        # b) the wrappers in the newer SDKs are bash scripts, which can cause problems
        # on windows, depending on how the wrapper is called.
        # Instead, we ensure that all the necessary flags for the c-compiler are set
        # via environment variables such as `TARGET_CFLAGS`.
        def to_sdk_llvm_bin(prog: str):
            if sys.platform == "win32":
                prog = prog + ".exe"
            llvm_prog = llvm_bin.joinpath(prog)
            if not llvm_prog.is_file():
                raise FileNotFoundError(errno.ENOENT, os.strerror(errno.ENOENT), llvm_prog)
            return llvm_bin.joinpath(prog).as_posix()

        # CC and CXX should already be set to appropriate host compilers by `build_env()`
        env["HOST_CC"] = env["CC"]
        env["HOST_CXX"] = env["CXX"]
        env["TARGET_AR"] = to_sdk_llvm_bin("llvm-ar")
        env["TARGET_RANLIB"] = to_sdk_llvm_bin("llvm-ranlib")
        env["TARGET_READELF"] = to_sdk_llvm_bin("llvm-readelf")
        env["TARGET_OBJCOPY"] = to_sdk_llvm_bin("llvm-objcopy")
        env["TARGET_STRIP"] = to_sdk_llvm_bin("llvm-strip")

        target_triple = self.triple()
        rust_target_triple = str(target_triple).replace("-", "_")
        ndk_clang = to_sdk_llvm_bin("clang")
        ndk_clangxx = to_sdk_llvm_bin("clang++")
        env[f"CC_{rust_target_triple}"] = ndk_clang
        env[f"CXX_{rust_target_triple}"] = ndk_clangxx
        # The clang target name is different from the LLVM target name
        clang_target_triple = str(target_triple).replace("-unknown-", "-")
        clang_target_triple_underscore = clang_target_triple.replace("-", "_")
        env[f"CC_{clang_target_triple_underscore}"] = ndk_clang
        env[f"CXX_{clang_target_triple_underscore}"] = ndk_clangxx
        # rustc linker
        env[f"CARGO_TARGET_{rust_target_triple.upper()}_LINKER"] = ndk_clang

        link_args = ["-fuse-ld=lld", f"--target={clang_target_triple}", f"--sysroot={ohos_sysroot_posix}"]

        env["HOST_CFLAGS"] = ""
        env["HOST_CXXFLAGS"] = ""
        ohos_cflags = [
            "-D__MUSL__",
            f" --target={clang_target_triple}",
            f" --sysroot={ohos_sysroot_posix}",
            "-Wno-error=unused-command-line-argument",
        ]
        if clang_target_triple.startswith("armv7-"):
            ohos_cflags.extend(["-march=armv7-a", "-mfloat-abi=softfp", "-mtune=generic-armv7-a", "-mthumb"])
        ohos_cflags_str = " ".join(ohos_cflags)
        env["TARGET_CFLAGS"] = ohos_cflags_str
        env["TARGET_CPPFLAGS"] = "-D__MUSL__"
        env["TARGET_CXXFLAGS"] = ohos_cflags_str

        # CMake related flags
        env["CMAKE"] = ndk_root.joinpath("build-tools", "cmake", "bin", "cmake").as_posix()
        cmake_toolchain_file = ndk_root.joinpath("build", "cmake", "ohos.toolchain.cmake")
        if cmake_toolchain_file.is_file():
            env[f"CMAKE_TOOLCHAIN_FILE_{rust_target_triple}"] = cmake_toolchain_file.as_posix()
        else:
            print(
                f"Warning: Failed to find the OpenHarmony CMake Toolchain file - Expected it at {cmake_toolchain_file}"
            )
        env[f"CMAKE_C_COMPILER_{rust_target_triple}"] = ndk_clang
        env[f"CMAKE_CXX_COMPILER_{rust_target_triple}"] = ndk_clangxx

        # pkg-config
        pkg_config_path = "{}:{}".format(
            ohos_sysroot.joinpath("usr", "lib", "pkgconfig").as_posix(),
            ohos_sysroot.joinpath("usr", "share", "pkgconfig").as_posix(),
        )
        env[f"PKG_CONFIG_SYSROOT_DIR_{rust_target_triple}"] = ohos_sysroot_posix
        env[f"PKG_CONFIG_PATH_{rust_target_triple}"] = pkg_config_path

        # bindgen / libclang-sys
        env["LIBCLANG_PATH"] = path.join(llvm_toolchain, "lib")
        env["CLANG_PATH"] = ndk_clangxx
        env[f"CXXSTDLIB_{clang_target_triple_underscore}"] = "c++"
        bindgen_extra_clangs_args_var = f"BINDGEN_EXTRA_CLANG_ARGS_{rust_target_triple}"
        bindgen_extra_clangs_args = env.get(bindgen_extra_clangs_args_var, "")
        bindgen_extra_clangs_args = bindgen_extra_clangs_args + " " + ohos_cflags_str
        env[bindgen_extra_clangs_args_var] = bindgen_extra_clangs_args

        # On OpenHarmony we add some additional flags when asan is enabled
        if config["build"]["sanitizer"] == SanitizerKind.ASAN:
            # Lookup `<sdk>/native/llvm/lib/clang/15.0.4/lib/aarch64-linux-ohos/libclang_rt.asan.so`
            lib_clang = llvm_toolchain.joinpath("lib", "clang")
            children = [f.path for f in os.scandir(lib_clang) if f.is_dir()]
            if len(children) != 1:
                raise RuntimeError(f"Expected exactly 1 libclang version: `{children}`")
            lib_clang_version_dir = pathlib.Path(children[0])
            libclang_arch = lib_clang_version_dir.joinpath("lib", clang_target_triple).resolve()
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

            # Use the clangrt from the NDK to use the same library for both C++ and Rust.
            env["RUSTFLAGS"] += " -Zexternal-clangrt"

            asan_compile_flags = (
                " -fsanitize=address -shared-libasan -fno-omit-frame-pointer -fsanitize-recover=address"
            )

            arch_asan_ignore_list = lib_clang_version_dir.joinpath("share", "asan_ignorelist.txt")
            if arch_asan_ignore_list.exists():
                asan_compile_flags += " -fsanitize-system-ignorelist=" + str(arch_asan_ignore_list)
            else:
                print(f"Warning: Couldn't find system ASAN ignorelist at `{arch_asan_ignore_list}`")
            env["TARGET_CFLAGS"] += asan_compile_flags
            env["TARGET_CXXFLAGS"] += asan_compile_flags

        link_args = [f"-Clink-arg={arg}" for arg in link_args]
        env["RUSTFLAGS"] += " " + " ".join(link_args)

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
