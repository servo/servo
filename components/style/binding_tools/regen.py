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

import regen_atoms

DESCRIPTION = 'Regenerate the rust version of the structs or the bindings file.'
TOOLS_DIR = os.path.dirname(os.path.abspath(__file__))
COMMON_BUILD_KEY = "__common__"

COMPILATION_TARGETS = {
    # Flags common for all the targets.
    COMMON_BUILD_KEY: {
        "flags": [
            "--no-unstable-rust", "--no-type-renaming",
        ],
        "clang_flags": [
            "-x", "c++", "-std=c++14",
            "-DTRACING=1", "-DIMPL_LIBXUL", "-DMOZ_STYLO_BINDINGS=1",
            "-DMOZILLA_INTERNAL_API", "-DRUST_BINDGEN", "-DMOZ_STYLO"
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
        "target_dir": "../gecko_bindings",
        "test": True,
        "flags": [
            "--ignore-functions",
            "--ignore-methods",
        ],
        "includes": [
            "{}/dist/include/nsThemeConstants.h",
            "{}/dist/include/mozilla/dom/AnimationEffectReadOnlyBinding.h",
            "{}/dist/include/mozilla/ServoElementSnapshot.h",
            "{}/dist/include/mozilla/dom/Element.h",
        ],
        "files": [
            "{}/dist/include/nsStyleStruct.h",
        ],
        "build_kinds": {
            "debug": {
                "clang_flags": [
                    "-DDEBUG=1",
                    "-DJS_DEBUG=1",
                ]
            },
            "release": {
            }
        },
        "raw_lines": [
            # We can get rid of this when the bindings move into the style crate.
            "pub enum OpaqueStyleData {}",
        ],
        "whitelist_vars": [
            "NS_THEME_.*",
            "NODE_.*",
            "NS_FONT_STYLE_.*",
            "NS_STYLE_.*",
            "NS_CORNER_.*",
            "NS_RADIUS_.*",
            "BORDER_COLOR_.*",
            "BORDER_STYLE_.*"
        ],
        "whitelist": [
            "RawGeckoNode",
            "RawGeckoElement",
            "RawGeckoDocument",
            "Element",
            "Side",
            "nsTArrayHeader",
            "nsCSSValueGradient",
            "nsCSSValueList_heap",
            "FrameRequestCallback",
            "nsCSSValueTriplet_heap",
            "nsCSSRect_heap",
            "AnonymousContent",
            "nsCSSValuePairList",
            "nsCSSValuePairList_heap",
            "nsCSSValuePair_heap",
            "CapturingContentInfo",
            "Runnable",
            "AudioContext",
            "FontFamilyListRefCnt",
            "ImageURL",
            "Image",
            "nsCSSValueFloatColor",
            "ServoAttrSnapshot",
            "GridNamedArea",
            "nsAttrName",
            "nsAttrValue",
            "nsCSSRect",
            "gfxFontFeature",
            "gfxAlternateValue",
            "nsCSSValueTokenStream",
            "nsSize",
            "pair",
            "StyleClipPathGeometryBox",
            "FontFamilyName",
            "nsCSSPropertyID",
            "StyleAnimation",
            "StyleTransition",
            "nsresult",
            "nsCSSValueGradientStop",
            "nsBorderColors",
            "Position",
            "nsCSSValueList",
            "nsCSSValue",
            "UniquePtr", "DefaultDelete",
            "StyleBasicShape",
            "nsMargin",
            "nsStyleContentData",
            "nsStyleFilter", "nsRect", "FragmentOrURL", "nsStyleCoord",
            "nsStyleCounterData", "StaticRefPtr", "nsTArray", "nsStyleFont",
            "nsStyleColor", "nsStyleList", "nsStyleText", "nsStyleVisibility",
            "nsStyleUserInterface", "nsStyleTableBorder", "nsStyleSVG",
            "nsStyleVariables", "nsStyleBackground", "nsStylePosition",
            "nsStyleTextReset", "nsStyleDisplay", "nsStyleContent",
            "nsStyleUIReset", "nsStyleTable", "nsStyleMargin",
            "nsStylePadding", "nsStyleBorder", "nsStyleOutline", "nsStyleXUL",
            "nsStyleSVGReset", "nsStyleColumn", "nsStyleEffects",
            "nsStyleImage", "nsStyleGradient", "nsStyleCoord",
            "nsStyleGradientStop", "nsStyleImageLayers",
            "nsStyleImageLayers_Layer", "nsStyleImageLayers_LayerType",
            "nsStyleUnit", "nsStyleUnion", "nsStyleCoord", "nsRestyleHint",
            "ServoElementSnapshot", "nsChangeHint", "SheetParsingMode",
            "nsMainThreadPtrHandle", "nsMainThreadPtrHolder", "nscolor",
            "nsFont", "FontFamilyList", "FontFamilyType", "nsIAtom",
            "nsStyleContext", "StyleClipPath", "StyleBasicShapeType",
            "StyleBasicShape", "nsCSSShadowArray",
        ],
        "opaque_types": [
            "atomic___base",
            "nsAString_internal_char_traits",
            "nsAString_internal_incompatible_char_type",
            "nsACString_internal_char_traits",
            "nsACString_internal_incompatible_char_type",
            "RefPtr_Proxy", "nsAutoPtr_Proxy", "Pair_Base",
            "RefPtr_Proxy_member_function", "nsAutoPtr_Proxy_member_function",
            "nsWritingIterator_reference", "nsReadingIterator_reference",
            "Heap", "TenuredHeap", "Rooted", "WeakPtr",  # <- More template magic than what
                                                         #    we support.
            "nsTObserverArray",  # <- Inherits from nsAutoTObserverArray<T, 0>
            "PLArenaPool",  # <- Bindgen bug
            "nsTHashtable",  # <- Inheriting from inner typedefs that clang
                             #    doesn't expose properly.
            "nsRefPtrHashtable", "nsDataHashtable", "nsClassHashtable",  # <- Ditto
            "nsIDocument_SelectorCache",  # <- Inherits from nsExpirationTracker<.., 4>
            "nsIPresShell_ScrollAxis",  # <- For some reason the alignment of this is 4
                                        # for clang.
            "nsPIDOMWindow",  # <- Takes the vtable from a template parameter, and we can't
                              #    generate it conditionally.
            "SupportsWeakPtr",
            "Maybe",  # <- AlignedStorage, which means templated union, which
                      #    means impossible to represent in stable rust as of
                      #    right now.
            "gfxSize",  # <- Same, union { struct { T width; T height; }; T components[2] };
            "gfxSize_Super",  # Ditto.
        ],
        "servo_mapped_generic_types": [
            {
                "generic": True,
                "gecko": "ServoUnsafeCell",
                "servo": "::std::cell::UnsafeCell"
            }, {
                "generic": True,
                "gecko": "ServoCell",
                "servo": "::std::cell::Cell"
            }, {
                "generic": False,
                "gecko": "ServoNodeData",
                "servo": "OpaqueStyleData"
            }
        ],
    },
    # Generation of the ffi bindings.
    "bindings": {
        "target_dir": "../gecko_bindings",
        "raw_lines": [
            "use heapsize::HeapSizeOf;",
        ],
        "flags": [
            "--ignore-methods",
        ],
        "match_headers": [
            "ServoBindingList.h",
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
            "nsStyleCoord", "nsStyleGradientStop", "nsStyleImageLayers",
            "nsStyleImageLayers_Layer", "nsStyleImageLayers_LayerType",
            "nsStyleUnit", "nsStyleUnion", "nsStyleCoord_CalcValue",
            "nsStyleCoord_Calc", "nsRestyleHint", "ServoElementSnapshot",
            "nsChangeHint", "SheetParsingMode", "nsMainThreadPtrHandle",
            "nsMainThreadPtrHolder", "nscolor", "nsFont", "FontFamilyList",
            "FontFamilyType", "nsIAtom", "nsStyleContext", "StyleClipPath",
            "StyleBasicShapeType", "StyleBasicShape", "nsCSSShadowArray",
            "nsINode", "nsIDocument", "nsIPrincipal", "nsIURI",
            "RawGeckoNode", "RawGeckoElement", "RawGeckoDocument",
            "ServoNodeData",
        ],
        "servo_nullable_arc_types": [
            "ServoComputedValues", "RawServoStyleSheet",
            "ServoDeclarationBlock"
        ],
        "servo_owned_types": [
            "RawServoStyleSet",
            "StyleChildrenIterator",
        ],
        "servo_immutable_borrow_types": [
            "RawGeckoNode",
            "RawGeckoElement",
            "RawGeckoDocument",
        ],
        "whitelist_functions": [
            "Servo_.*",
            "Gecko_.*"
        ]
    },

    "atoms": {
        "custom_build": regen_atoms.build,
    }
}


def platform_dependent_defines():
    ret = []

    if os.name == "posix":
        ret.append("-DOS_POSIX=1")

    system = platform.system()
    if system == "Linux":
        ret.append("-DOS_LINUX=1")
    elif system == "Darwin":
        ret.append("-DOS_MACOSX=1")
    elif system == "Windows":
        ret.append("-DOS_WIN=1")
        ret.append("-DWIN32=1")
        ret.append("-use-msvc-mangling")
        msvc_platform = os.environ["PLATFORM"]
        if msvc_platform == "X86":
            ret.append("--target=i686-pc-win32")
        elif msvc_platform == "X64":
            ret.append("--target=x86_64-pc-win32")
        else:
            raise Exception("Only MSVC builds are supported on Windows")
        # For compatibility with MSVC 2015
        ret.append("-fms-compatibility-version=19")
        # To enable the builtin __builtin_offsetof so that CRT wouldn't
        # use reinterpret_cast in offsetof() which is not allowed inside
        # static_assert().
        ret.append("-D_CRT_USE_BUILTIN_OFFSETOF")
        # Enable hidden attribute (which is not supported by MSVC and
        # thus not enabled by default with a MSVC-compatibile build)
        # to exclude hidden symbols from the generated file.
        ret.append("-DHAVE_VISIBILITY_HIDDEN_ATTRIBUTE=1")
    else:
        raise Exception("Unknown platform")

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


def build(objdir, target_name, debug, debugger, kind_name=None,
          output_filename=None, bindgen=None, skip_test=False,
          verbose=False):
    assert target_name in COMPILATION_TARGETS

    current_target = COMPILATION_TARGETS[target_name]
    if COMMON_BUILD_KEY in COMPILATION_TARGETS:
        current_target = copy.deepcopy(COMPILATION_TARGETS[COMMON_BUILD_KEY])
        extend_object(current_target, COMPILATION_TARGETS[target_name])

    assert ((kind_name is None and "build_kinds" not in current_target) or
            (kind_name in current_target["build_kinds"]))

    if "custom_build" in current_target:
        print("[CUSTOM] {}::{} in \"{}\"... ".format(target_name, kind_name, objdir), end='')
        sys.stdout.flush()
        ret = current_target["custom_build"](objdir, verbose=True)
        if ret != 0:
            print("FAIL")
        else:
            print("OK")

        return ret

    if bindgen is None:
        bindgen = os.path.join(TOOLS_DIR, "rust-bindgen")

    if os.path.isdir(bindgen):
        bindgen = ["cargo", "run", "--manifest-path",
                   os.path.join(bindgen, "Cargo.toml"), "--features", "llvm_stable", "--"]
    else:
        bindgen = [bindgen]

    if kind_name is not None:
        current_target = copy.deepcopy(current_target)
        extend_object(current_target, current_target["build_kinds"][kind_name])

    target_dir = None
    if output_filename is None and "target_dir" in current_target:
        target_dir = current_target["target_dir"]

    if output_filename is None:
        output_filename = "{}.rs".format(target_name)

        if kind_name is not None:
            output_filename = "{}_{}.rs".format(target_name, kind_name)

    if target_dir:
        output_filename = "{}/{}".format(target_dir, output_filename)

    print("[BINDGEN] {}::{} in \"{}\"... ".format(target_name, kind_name, objdir), end='')
    sys.stdout.flush()

    flags = []

    # This makes an FFI-safe void type that can't be matched on
    # &VoidType is UB to have, because you can match on it
    # to produce a reachable unreachable. If it's wrapped in
    # a struct as a private field it becomes okay again
    #
    # Not 100% sure of how safe this is, but it's what we're using
    # in the XPCOM ffi too
    # https://github.com/nikomatsakis/rust-memory-model/issues/2
    def zero_size_type(ty, flags):
        flags.append("--blacklist-type")
        flags.append(ty)
        flags.append("--raw-line")
        flags.append("enum {0}Void{{ }}".format(ty))
        flags.append("--raw-line")
        flags.append("pub struct {0}({0}Void);".format(ty))

    if "flags" in current_target:
        flags.extend(current_target["flags"])

    clang_flags = []

    if "clang_flags" in current_target:
        clang_flags.extend(current_target["clang_flags"])

    clang_flags.extend(platform_dependent_defines())

    if "raw_lines" in current_target:
        for raw_line in current_target["raw_lines"]:
            flags.append("--raw-line")
            flags.append(raw_line)

    if "search_dirs" in current_target:
        for dir_name in current_target["search_dirs"]:
            clang_flags.append("-I")
            clang_flags.append(dir_name.format(objdir))

    if "includes" in current_target:
        for file_name in current_target["includes"]:
            clang_flags.append("-include")
            clang_flags.append(file_name.format(objdir))

    if "whitelist" in current_target:
        for header in current_target["whitelist"]:
            flags.append("--whitelist-type")
            flags.append(header)

    if "whitelist_functions" in current_target:
        for header in current_target["whitelist_functions"]:
            flags.append("--whitelist-function")
            flags.append(header)

    if "whitelist_vars" in current_target:
        for header in current_target["whitelist_vars"]:
            flags.append("--whitelist-var")
            flags.append(header)

    if "opaque_types" in current_target:
        for ty in current_target["opaque_types"]:
            flags.append("--opaque-type")
            flags.append(ty)

    if "servo_nullable_arc_types" in current_target:
        for ty in current_target["servo_nullable_arc_types"]:
            flags.append("--blacklist-type")
            flags.append("{}Strong".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}Strong = ::gecko_bindings::sugar::ownership::Strong<{0}>;"
                         .format(ty))
            flags.append("--blacklist-type")
            flags.append("{}BorrowedOrNull".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}BorrowedOrNull<'a> = \
::gecko_bindings::sugar::ownership::Borrowed<'a, {0}>;".format(ty))
            flags.append("--blacklist-type")
            flags.append("{}Borrowed".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}Borrowed<'a> = &'a {0};".format(ty))
            zero_size_type(ty, flags)

    if "servo_immutable_borrow_types" in current_target:
        for ty in current_target["servo_immutable_borrow_types"]:
            flags.append("--blacklist-type")
            flags.append("{}Borrowed".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}Borrowed<'a> = &'a {0};".format(ty))
            flags.append("--blacklist-type")
            flags.append("{}BorrowedOrNull".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}BorrowedOrNull<'a> = \
::gecko_bindings::sugar::ownership::Borrowed<'a, {0}>;".format(ty))
            # Right now the only immutable borrow types are ones which we import
            # from the |structs| module. As such, we don't need to create an opaque
            # type with zero_size_type. If we ever introduce immutable borrow types
            # which _do_ need to be opaque, we'll need a separate mode.

    if "servo_mapped_generic_types" in current_target:
        for ty in current_target["servo_mapped_generic_types"]:
            flags.append("--blacklist-type")
            flags.append("{}".format(ty["gecko"]))
            flags.append("--raw-line")
            flags.append("pub type {0}{2} = {1}{2};".format(ty["gecko"], ty["servo"], "<T>" if ty["generic"] else ""))

    if "servo_owned_types" in current_target:
        for ty in current_target["servo_owned_types"]:
            flags.append("--blacklist-type")
            flags.append("{}Borrowed".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}Borrowed<'a> = &'a {0};".format(ty))
            flags.append("--blacklist-type")
            flags.append("{}BorrowedMut".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}BorrowedMut<'a> = &'a mut {0};".format(ty))
            flags.append("--blacklist-type")
            flags.append("{}Owned".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}Owned = ::gecko_bindings::sugar::ownership::Owned<{0}>;".format(ty))
            flags.append("--blacklist-type")
            flags.append("{}BorrowedOrNull".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}BorrowedOrNull<'a> = ::gecko_bindings::sugar::ownership::Borrowed<'a, {0}>;"
                         .format(ty))
            flags.append("--blacklist-type")
            flags.append("{}BorrowedMutOrNull".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}BorrowedMutOrNull<'a> = ::gecko_bindings::sugar::ownership::BorrowedMut<'a, {0}>;"
                         .format(ty))
            flags.append("--blacklist-type")
            flags.append("{}OwnedOrNull".format(ty))
            flags.append("--raw-line")
            flags.append("pub type {0}OwnedOrNull = ::gecko_bindings::sugar::ownership::OwnedOrNull<{0}>;".format(ty))
            zero_size_type(ty, flags)

    if "structs_types" in current_target:
        for ty in current_target["structs_types"]:
            flags.append("--blacklist-type")
            flags.append(ty)
            flags.append("--raw-line")
            flags.append("use gecko_bindings::structs::{};".format(ty))

            # TODO: this is hacky, figure out a better way to do it without
            # hardcoding everything...
            if ty.startswith("nsStyle"):
                flags.extend([
                    "--raw-line",
                    "unsafe impl Send for {} {{}}".format(ty),
                    "--raw-line",
                    "unsafe impl Sync for {} {{}}".format(ty),
                ])

    flags.append("-o")
    flags.append(output_filename)

    assert len(current_target["files"]) == 1
    flags.append(current_target["files"][0].format(objdir))

    flags = bindgen + flags + ["--"] + clang_flags

    if verbose:
        print(flags)

    output = ""
    try:
        if debug:
            flags = [debugger, "--args"] + flags
            subprocess.check_call(flags)
        else:
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

        with tempfile.NamedTemporaryFile(delete=False) as f:
            test_file = f.name
        output = None
        try:
            rustc_command = ["rustc", output_filename, "--test", "-o", test_file]
            output = subprocess.check_output(rustc_command, stderr=subprocess.STDOUT)
            output = output.decode('utf8')
        except subprocess.CalledProcessError as e:
            print("FAIL\n", e.output.decode('utf8'))
            return 1

        print("OK")

        if verbose:
            print(output)

        print("[RUSTC_TEST]... ", end='')
        sys.stdout.flush()

        try:
            output = subprocess.check_output([test_file], stderr=subprocess.STDOUT)
            output = output.decode('utf8')
        except subprocess.CalledProcessError as e:
            print("tests failed: ", e.output.decode('utf8'))
            return 1

        os.remove(test_file)
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
    parser.add_argument('--target', default='all',
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
    parser.add_argument('--debug',
                        action='store_true',
                        help='Try to use a debugger to debug bindgen commands (default: gdb)')
    parser.add_argument('--debugger', default='gdb',
                        help='Debugger to use. Only used if --debug is passed.')
    parser.add_argument('objdir')

    args = parser.parse_args()

    if not os.path.isdir(args.objdir):
        print("\"{}\" doesn't seem to be a directory".format(args.objdir))
        return 1

    if (args.target != "all" and args.target not in COMPILATION_TARGETS) or args.target == COMMON_BUILD_KEY:
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
        ret = build(args.objdir, target, kind_name=kind,
                    debug=args.debug, debugger=args.debugger,
                    bindgen=args.bindgen, skip_test=args.skip_test,
                    output_filename=args.output,
                    verbose=args.verbose)
        if ret != 0:
            print("{}::{} failed".format(target, kind))
            return ret

    return 0

if __name__ == '__main__':
    sys.exit(main())
