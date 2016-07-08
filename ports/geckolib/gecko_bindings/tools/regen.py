#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function
import os
import sys
import argparse
import platform
import copy
import subprocess
import tempfile

DESCRIPTION = 'Regenerate the rust version of the structs or the bindings file.'
TOOLS_DIR = os.path.dirname(os.path.abspath(__file__))
COMMON_BUILD_KEY = "__common__"

COMPILATION_TARGETS = {
    # Flags common for all the targets.
    COMMON_BUILD_KEY: {
        "flags": [
            "-x", "c++", "-std=gnu++0x",
            "-allow-unknown-types", "-no-bitfield-methods",
            "-no-type-renaming", "-no-namespaced-constants",
            "-DTRACING=1", "-DIMPL_LIBXUL", "-DMOZ_STYLO_BINDINGS=1",
            "-DMOZILLA_INTERNAL_API",
        ],
        "search_dirs": [
            "{}/dist/include",
            "{}/dist/include/nspr",
            "{}/../nsprpub/pr/include"
        ],
        "includes": [
            "{}/mozilla-config.h",
        ],
    },
    # Generation of style structs.
    "structs": {
        "test": True,
        "flags": [
            "-ignore-functions",
        ],
        "includes": [
            "{}/dist/include/nsThemeConstants.h",
            "{}/dist/include/mozilla/dom/AnimationEffectReadOnlyBinding.h",
        ],
        "files": [
            "{}/dist/include/nsStyleStruct.h",
        ],
        "build_kinds": {
            "debug": {
                "flags": [
                    "-DDEBUG=1",
                    "-DJS_DEBUG=1",
                ]
            },
            "release": {
            }
        },
        "match_headers": [
            "RefCountType.h", "nscore.h", "nsError.h", "nsID.h", "nsString",
            "nsAString", "nsSubstring", "nsTSubstring", "nsTString",
            "nsISupportsBase.h", "nsCOMPtr.h", "nsIAtom.h", "nsIURI.h",
            "nsAutoPtr.h", "nsColor.h", "nsCoord.h", "nsPoint.h", "nsRect.h",
            "nsMargin.h", "nsThemeConstants.h", "nsCSSProperty.h",
            "CSSVariableValues.h", "nsFont.h", "nsTHashtable.h",
            "PLDHashTable.h", "nsColor.h", "nsStyleStruct.h", "nsStyleCoord.h",
            "RefPtr.h", "nsISupportsImpl.h", "gfxFontConstants.h",
            "gfxFontFamilyList.h", "gfxFontFeatures.h", "imgRequestProxy.h",
            "nsIRequest.h", "imgIRequest.h", "CounterStyleManager.h",
            "nsStyleConsts.h", "nsCSSValue.h", "SheetType.h", "nsIPrincipal.h",
            "nsDataHashtable.h", "nsCSSScanner.h", "utility", "nsTArray",
            "pair", "SheetParsingMode.h", "StaticPtr.h", "nsProxyRelease.h",
            "mozilla/dom/AnimationEffectReadOnlyBinding.h",
            "/Types.h",  # <- Disallow UnionTypes.h
        ],
        "blacklist": [
            "IsDestructibleFallbackImpl", "IsDestructibleFallback",
            "ProxyReleaseEvent", "FallibleTArray", "nsTArray_Impl",
            "__is_tuple_like_impl", "tuple_size", "tuple",
            "__make_pair_return_impl", "__make_pair_return", "tuple_element",
            "_Itup_cat", "AnimationEffectTimingProperties",
            "FastAnimationEffectTimingProperties", "ComputedTimingProperties",
            "FastComputedTimingProperties",
        ],
        "opaque_types": [
            "nsIntMargin", "nsIntPoint", "nsIntRect", "nsCOMArray",
            "nsDependentString", "EntryStore", "gfxFontFeatureValueSet",
            "imgRequestProxy", "imgRequestProxyStatic", "CounterStyleManager",
            "ImageValue", "URLValue", "URLValueData", "nsIPrincipal",
            "nsDataHashtable", "imgIRequest"
        ]
    },
    # Generation of the ffi bindings.
    "bindings": {
        "raw_lines": [
            "use heapsize::HeapSizeOf;",
        ],
        "match_headers": [
            "ServoBindings.h",
            "nsStyleStructList.h",
        ],
        "files": [
            "{}/dist/include/mozilla/ServoBindings.h",
        ],

        # Types to just use from the `structs` target.
        "structs_types": [
            "nsStyleFont", "nsStyleColor", "nsStyleList", "nsStyleText",
            "nsStyleVisibility", "nsStyleUserInterface", "nsStyleTableBorder",
            "nsStyleSVG", "nsStyleVariables", "nsStyleBackground",
            "nsStylePosition", "nsStyleTextReset", "nsStyleDisplay",
            "nsStyleContent", "nsStyleUIReset", "nsStyleTable",
            "nsStyleMargin", "nsStylePadding", "nsStyleBorder",
            "nsStyleOutline", "nsStyleXUL", "nsStyleSVGReset", "nsStyleColumn",
            "nsStyleEffects", "nsStyleImage", "nsStyleGradient",
            "nsStyleCoord", "nsStyleGradientStop",

            "SheetParsingMode", "nsMainThreadPtrHandle",
            "nsMainThreadPtrHolder", "nscolor", "nsFont", "FontFamilyList",
            "FontFamilyType", "nsIAtom",
        ],
    }
}


def platform_dependent_defines():
    ret = []

    if os.name == "posix":
        ret.append("-DOS_POSIX=1")

    ret.append({
        "Linux": "-DOS_LINUX=1",
        "Darwin": "-DOS_MACOSX=1",
        # TODO: Windows?
    }[platform.system()])

    return ret


def extend_object(obj, other):
    if not obj or not other:
        return obj

    if isinstance(obj, list) and isinstance(other, list):
        obj.extend(other)
        return

    assert isinstance(obj, dict) and isinstance(other, dict)

    for key in other.keys():
        if key in obj:
            extend_object(obj[key], other[key])
        else:
            obj[key] = copy.deepcopy(other[key])


def build(objdir, target_name, kind_name=None,
          output_filename=None, bindgen=None, skip_test=False,
          verbose=False):
    assert target_name in COMPILATION_TARGETS

    current_target = COMPILATION_TARGETS[target_name]
    if COMMON_BUILD_KEY in COMPILATION_TARGETS:
        current_target = copy.deepcopy(COMPILATION_TARGETS[COMMON_BUILD_KEY])
        extend_object(current_target, COMPILATION_TARGETS[target_name])

    assert ((kind_name is None and "build_kinds" not in current_target) or
            (kind_name in current_target["build_kinds"]))

    if bindgen is None:
        bindgen = "{}/rust-bindgen/target/debug/bindgen".format(TOOLS_DIR)

    if output_filename is None:
        filename = "{}.rs".format(target_name)

        if kind_name is not None:
            filename = "{}_{}.rs".format(target_name, kind_name)

        output_filename = "{}/../{}".format(TOOLS_DIR, filename)

    if kind_name is not None:
        current_target = copy.deepcopy(current_target)
        extend_object(current_target, current_target["build_kinds"][kind_name])

    print("[BINDGEN] {}::{} in \"{}\"... ".format(target_name, kind_name, objdir), end='')
    sys.stdout.flush()

    flags = []
    flags.extend(platform_dependent_defines())

    if "flags" in current_target:
        flags.extend(current_target["flags"])

    if "raw_lines" in current_target:
        for raw_line in current_target["raw_lines"]:
            flags.append("-raw-line")
            flags.append(raw_line)

    if "search_dirs" in current_target:
        for dir_name in current_target["search_dirs"]:
            flags.append("-I")
            flags.append(dir_name.format(objdir))

    if "includes" in current_target:
        for file_name in current_target["includes"]:
            flags.append("-include")
            flags.append(file_name.format(objdir))

    if "match_headers" in current_target:
        for header in current_target["match_headers"]:
            flags.append("-match")
            flags.append(header.format(objdir))

    if "blacklist" in current_target:
        for ty in current_target["blacklist"]:
            flags.append("-blacklist-type")
            flags.append(ty)

    if "opaque_types" in current_target:
        for ty in current_target["opaque_types"]:
            flags.append("-opaque-type")
            flags.append(ty)

    if "structs_types" in current_target:
        for ty in current_target["structs_types"]:
            flags.append("-blacklist-type")
            flags.append(ty)
            flags.append("-raw-line")
            flags.append("use structs::{};".format(ty))
            # TODO: this is hacky, figure out a better way to do it without
            # hardcoding everything...
            if ty.startswith("nsStyle"):
                flags.extend([
                    "-raw-line",
                    "unsafe impl Send for {} {{}}".format(ty),
                    "-raw-line",
                    "unsafe impl Sync for {} {{}}".format(ty),
                    "-raw-line",
                    "impl HeapSizeOf for {} {{ fn heap_size_of_children(&self) -> usize {{ 0 }} }}".format(ty)
                ])

    flags.append("-o")
    flags.append(output_filename)

    # TODO: support more files, that's the whole point of this.
    assert len(current_target["files"]) == 1
    flags.append(current_target["files"][0].format(objdir))

    flags.insert(0, bindgen)
    output = None
    try:
        output = subprocess.check_output(flags, stderr=subprocess.STDOUT)
        output = output.decode('utf8')
    except subprocess.CalledProcessError as e:
        print("FAIL\n", e.output.decode('utf8'))
        return 1

    print("OK")

    if verbose:
        print(output)

    if current_target.get("test", False) and not skip_test:
        print("[RUSTC]... ", end='')
        sys.stdout.flush()

        tests_file = tempfile.NamedTemporaryFile()
        output = None
        try:
            rustc_command = ["rustc", output_filename, "--test", "-o", tests_file.name]
            output = subprocess.check_output(rustc_command, stderr=subprocess.STDOUT)
            output = output.decode('utf8')
        except subprocess.CalledProcessError as e:
            print("FAIL\n", e.output.decode('utf8'))
            return 1

        print("OK")

        if verbose:
            print(output)

        tests_file.file.close()
        print("[RUSTC_TEST]... ", end='')
        sys.stdout.flush()

        try:
            output = subprocess.check_output([tests_file.name], stderr=subprocess.STDOUT)
            output = output.decode('utf8')
        except subprocess.CalledProcessError as e:
            print("tests failed: ", e.output.decode('utf8'))
            return 1

        print("OK")

        # TODO: this -3 is hacky as heck
        print(output.split('\n')[-3])

        if verbose:
            print(output)

    return 0


def builds_for(target_name, kind):
    if target_name == "all":
        for target in COMPILATION_TARGETS.keys():
            if target == COMMON_BUILD_KEY:
                continue

            if "build_kinds" in COMPILATION_TARGETS[target]:
                for kind in COMPILATION_TARGETS[target]["build_kinds"].keys():
                    yield (target, kind)
            else:
                yield (target, None)
        return

    target = COMPILATION_TARGETS[target_name]
    if "build_kinds" in target:
        if kind is None:
            for kind in target["build_kinds"].keys():
                yield(target_name, kind)
        else:
            yield (target_name, kind)
        return

    yield (target_name, None)


def main():
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    parser.add_argument('--target',
                        help='The target to build, either "structs" or "bindings"')
    parser.add_argument('--kind',
                        help='Kind of build')
    parser.add_argument('--bindgen',
                        help='Override bindgen binary')
    parser.add_argument('--output', '-o',
                        help='Output of the script')
    parser.add_argument('--skip-test',
                        action='store_true',
                        help='Skip automatic tests, useful for debugging')
    parser.add_argument('--verbose', '-v',
                        action='store_true',
                        help='Be... verbose')
    parser.add_argument('objdir')

    args = parser.parse_args()

    if not os.path.isdir(args.objdir):
        print("\"{}\" doesn't seem to be a directory".format(args.objdir))
        return 1

    if args.target != COMMON_BUILD_KEY and args.target != "all" and args.target not in COMPILATION_TARGETS:
        print("{} is not a valid compilation target.".format(args.target))
        print("Valid compilation targets are:")
        for target in COMPILATION_TARGETS.keys():
            if target != COMMON_BUILD_KEY:
                print("\t * {}".format(target))
        return 1

    current_target = COMPILATION_TARGETS.get(args.target, {})
    if args.kind and "build_kinds" in current_target and args.kind not in current_target["build_kinds"]:
        print("{} is not a valid build kind.".format(args.kind))
        print("Valid build kinds are:")
        for kind in current_target["build_kinds"].keys():
            print("\t * {}".format(kind))
        return 1

    for target, kind in builds_for(args.target, args.kind):
        ret = build(args.objdir, target, kind,
                    bindgen=args.bindgen, skip_test=args.skip_test,
                    output_filename=args.output,
                    verbose=args.verbose)
        if ret != 0:
            print("{}::{} failed".format(target, kind))
            return ret

    return 0

if __name__ == '__main__':
    sys.exit(main())
