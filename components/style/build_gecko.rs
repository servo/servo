/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libbindgen::Builder;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

lazy_static! {
    static ref INCLUDE_RE: Regex = Regex::new(r#"#include\s*"(.+?)""#).unwrap();
    static ref OBJDIR: Option<String> = env::var("MOZ_OBJDIR").ok();
    static ref OBJDIR_PATH: &'static Path = Path::new(OBJDIR.as_ref().unwrap());
    static ref OUTDIR: String = env::var("OUT_DIR").unwrap();
    static ref OUTDIR_PATH: &'static Path = Path::new(&*OUTDIR);
    static ref SEARCH_PATHS: Vec<PathBuf> = [
        "dist/include", "dist/include/nspr"
    ].iter().map(|dir| OBJDIR_PATH.join(dir)).collect();
    static ref ADDED_PATHS: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
}

fn search_include(name: &str) -> Option<PathBuf> {
    for path in SEARCH_PATHS.iter() {
        let file = path.join(name);
        if file.is_file() {
            return Some(file);
        }
    }
    None
}

fn add_headers_recursively(path: PathBuf, added_paths: &mut HashSet<PathBuf>) {
    if added_paths.contains(&path) {
        return;
    }
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    added_paths.insert(path);
    // Find all includes and add them recursively
    for cap in INCLUDE_RE.captures_iter(&content) {
        if let Some(path) = search_include(cap.at(1).unwrap()) {
            add_headers_recursively(path, added_paths);
        }
    }
}

fn add_include(name: &str) -> String {
    let mut added_paths = ADDED_PATHS.lock().unwrap();
    let file = search_include(name).unwrap();
    let result = String::from(file.to_str().unwrap());
    add_headers_recursively(file, &mut *added_paths);
    result
}

trait BuilderExt {
    fn get_initial_builder() -> Builder;
    fn include<T: Into<String>>(self, file: T) -> Builder;
    fn zero_size_type(self, ty: &str) -> Builder;
    fn borrowed_type(self, ty: &str) -> Builder;
    fn mutable_borrowed_type(self, ty: &str) -> Builder;
    fn whitelisted_types(mut self, iter: &[&str]) -> Builder;
    fn whitelisted_vars(mut self, iter: &[&str]) -> Builder;
    fn opaque_types(mut self, list: &[&str]) -> Builder;
}

impl BuilderExt for Builder {
    fn get_initial_builder() -> Builder {
        let mut builder = Builder::default().no_unstable_rust();
        let args = [
            "-x", "c++", "-std=c++14",
            "-DTRACING=1", "-DIMPL_LIBXUL", "-DMOZ_STYLO_BINDINGS=1",
            "-DMOZILLA_INTERNAL_API", "-DRUST_BINDGEN", "-DMOZ_STYLO"
        ];
        for &arg in args.iter() {
            builder = builder.clang_arg(arg);
        }
        for dir in SEARCH_PATHS.iter() {
            builder = builder.clang_arg("-I").clang_arg(dir.to_str().unwrap());
        }
        builder = builder.include(OBJDIR_PATH.join("mozilla-config.h")
                                  .into_os_string().into_string().unwrap());

        if cfg!(debug_assertions) {
            builder = builder.clang_arg("-DDEBUG=1").clang_arg("-DJS_DEBUG=1");
        }
        if cfg!(target_family = "unix") {
            builder = builder.clang_arg("-DOS_POSIX=1");
        }
        if cfg!(target_os = "linux") {
            builder = builder.clang_arg("-DOS_LINUX=1");
        } else if cfg!(target_os = "macos") {
            builder = builder.clang_arg("-DOS_MACOSX=1");
        } else if cfg!(target_env = "msvc") {
            builder = builder.clang_arg("-DOS_WIN=1").clang_arg("-DWIN32=1")
                // For compatibility with MSVC 2015
                .clang_arg("-fms-compatibility-version=19")
                // To enable the builtin __builtin_offsetof so that CRT wouldn't
                // use reinterpret_cast in offsetof() which is not allowed inside
                // static_assert().
                .clang_arg("-D_CRT_USE_BUILTIN_OFFSETOF")
                // Enable hidden attribute (which is not supported by MSVC and
                // thus not enabled by default with a MSVC-compatibile build)
                // to exclude hidden symbols from the generated file.
                .clang_arg("-DHAVE_VISIBILITY_HIDDEN_ATTRIBUTE=1");
            if cfg!(target_pointer_width = "32") {
                builder = builder.clang_arg("--target=i686-pc-win32");
            } else {
                builder = builder.clang_arg("--target=x86_64-pc-win32");
            }
        } else {
            panic!("Unknown platform");
        }
        builder
    }
    fn include<T: Into<String>>(self, file: T) -> Builder {
        self.clang_arg("-include").clang_arg(file)
    }
    // This makes an FFI-safe void type that can't be matched on
    // &VoidType is UB to have, because you can match on it
    // to produce a reachable unreachable. If it's wrapped in
    // a struct as a private field it becomes okay again
    //
    // Not 100% sure of how safe this is, but it's what we're using
    // in the XPCOM ffi too
    // https://github.com/nikomatsakis/rust-memory-model/issues/2
    fn zero_size_type(self, ty: &str) -> Builder {
        self.hide_type(ty)
            .raw_line(format!("enum {}Void {{ }}", ty))
            .raw_line(format!("pub struct {0}({0}Void);", ty))
    }
    fn borrowed_type(self, ty: &str) -> Builder {
        self.hide_type(format!("{}Borrowed", ty))
            .raw_line(format!("pub type {0}Borrowed<'a> = &'a {0};", ty))
            .hide_type(format!("{}BorrowedOrNull", ty))
            .raw_line(format!("pub type {0}BorrowedOrNull<'a> = Option<&'a {0}>;", ty))
    }
    fn mutable_borrowed_type(self, ty: &str) -> Builder {
        self.borrowed_type(ty)
            .hide_type(format!("{}BorrowedMut", ty))
            .raw_line(format!("pub type {0}BorrowedMut<'a> = &'a mut {0};", ty))
            .hide_type(format!("{}BorrowedMutOrNull", ty))
            .raw_line(format!("pub type {0}BorrowedMutOrNull<'a> = Option<&'a mut {0}>;", ty))
    }
    fn whitelisted_types(mut self, list: &[&str]) -> Builder {
        for &ty in list.into_iter() {
            self = self.whitelisted_type(ty);
        }
        self
    }
    fn whitelisted_vars(mut self, list: &[&str]) -> Builder {
        for &var in list.into_iter() {
            self = self.whitelisted_var(var);
        }
        self
    }
    fn opaque_types(mut self, list: &[&str]) -> Builder {
        for &ty in list.into_iter() {
            self = self.opaque_type(ty);
        }
        self
    }
}

const GECKO_BINDINGS: &'static str = "gecko_bindings.rs";
const GECKO_STRUCTS: &'static str = "gecko_structs.rs";

fn write_binding_file(builder: Builder, file: &str) {
    let bindings = builder.generate().expect("Unable to generate bindings");
    let binding_file = File::create(&OUTDIR_PATH.join(file)).unwrap();
    bindings.write(Box::new(BufWriter::new(binding_file))).expect("Unable to write output");
}

fn generate_bindings() {
    let mut builder = Builder::get_initial_builder()
        .ignore_methods()
        .header(add_include("mozilla/ServoBindings.h"))
        .hide_type("nsACString_internal")
        .hide_type("nsAString_internal")
        .raw_line("pub use nsstring::{nsACString, nsAString};")
        .raw_line("type nsACString_internal = nsACString;")
        .raw_line("type nsAString_internal = nsAString;")
        .whitelisted_function("Servo_.*")
        .whitelisted_function("Gecko_.*")
        .whitelisted_types(&[
            "RawGeckoDocument",
            "RawGeckoElement",
            "RawGeckoNode",
            "ThreadSafe.*Holder"
        ]);
    let structs_types = [
        "Element",
        "FontFamilyList",
        "FontFamilyType",
        "ServoElementSnapshot",
        "SheetParsingMode",
        "StyleBasicShape",
        "StyleBasicShapeType",
        "StyleClipPath",
        "nscoord",
        "nsCSSKeyword",
        "nsCSSShadowArray",
        "nsCSSValue",
        "nsCSSValueSharedList",
        "nsChangeHint",
        "nsFont",
        "nsIAtom",
        "nsIDocument",
        "nsINode",
        "nsIPrincipal",
        "nsIURI",
        "nsMainThreadPtrHolder",
        "nsRestyleHint",
        "nsString",
        "nsStyleBackground",
        "nsStyleBorder",
        "nsStyleColor",
        "nsStyleColumn",
        "nsStyleContent",
        "nsStyleContext",
        "nsStyleCoord",
        "nsStyleCoord_Calc",
        "nsStyleCoord_CalcValue",
        "nsStyleDisplay",
        "nsStyleEffects",
        "nsStyleFont",
        "nsStyleGradient",
        "nsStyleGradientStop",
        "nsStyleImage",
        "nsStyleImageLayers",
        "nsStyleImageLayers_Layer",
        "nsStyleImageLayers_LayerType",
        "nsStyleImageRequest",
        "nsStyleList",
        "nsStyleMargin",
        "nsStyleOutline",
        "nsStylePadding",
        "nsStylePosition",
        "nsStyleQuoteValues",
        "nsStyleSVG",
        "nsStyleSVGReset",
        "nsStyleTable",
        "nsStyleTableBorder",
        "nsStyleText",
        "nsStyleTextReset",
        "nsStyleUIReset",
        "nsStyleUnion",
        "nsStyleUnit",
        "nsStyleUserInterface",
        "nsStyleVariables",
        "nsStyleVisibility",
        "nsStyleXUL",
    ];
    let servo_nullable_arc_types = [
        "ServoComputedValues", "RawServoStyleSheet",
        "RawServoDeclarationBlock"
    ];
    let servo_owned_types = [
        "RawServoStyleSet",
        "StyleChildrenIterator",
    ];
    let servo_immutable_borrow_types = [
        "RawGeckoNode",
        "RawGeckoElement",
        "RawGeckoDocument",
        "RawServoDeclarationBlockStrong",
    ];
    let servo_borrow_types = [
        "nsCSSValue",
    ];
    for &ty in structs_types.iter() {
        builder = builder.hide_type(ty)
            .raw_line(format!("use gecko_bindings::structs::{};", ty));
        // TODO this is hacky, figure out a better way to do it without
        // hardcoding everything...
        if ty.starts_with("nsStyle") {
            builder = builder
                .raw_line(format!("unsafe impl Send for {} {{}}", ty))
                .raw_line(format!("unsafe impl Sync for {} {{}}", ty));
        }
    }
    for &ty in servo_nullable_arc_types.iter() {
        builder = builder
            .hide_type(format!("{}Strong", ty))
            .raw_line(format!("pub type {0}Strong = ::gecko_bindings::sugar::ownership::Strong<{0}>;", ty))
            .borrowed_type(ty)
            .zero_size_type(ty);
    }
    for &ty in servo_owned_types.iter() {
        builder = builder
            .hide_type(format!("{}Owned", ty))
            .raw_line(format!("pub type {0}Owned = ::gecko_bindings::sugar::ownership::Owned<{0}>;", ty))
            .hide_type(format!("{}OwnedOrNull", ty))
            .raw_line(format!("pub type {0}OwnedOrNull = ::gecko_bindings::sugar::ownership::OwnedOrNull<{0}>;", ty))
            .mutable_borrowed_type(ty)
            .zero_size_type(ty);
    }
    for &ty in servo_immutable_borrow_types.iter() {
        builder = builder.borrowed_type(ty);
    }
    for &ty in servo_borrow_types.iter() {
        builder = builder.mutable_borrowed_type(ty);
        // Right now the only immutable borrow types are ones which we import
        // from the |structs| module. As such, we don't need to create an opaque
        // type with zero_size_type. If we ever introduce immutable borrow types
        // which _do_ need to be opaque, we'll need a separate mode.
    }
    write_binding_file(builder, GECKO_BINDINGS);
}

fn generate_structs() {
    let mut builder = Builder::get_initial_builder()
        .ignore_functions()
        .ignore_methods()
        .header(add_include("nsStyleStruct.h"))
        .include(add_include("gfxFontConstants.h"))
        .include(add_include("nsThemeConstants.h"))
        .include(add_include("mozilla/dom/AnimationEffectReadOnlyBinding.h"))
        .include(add_include("mozilla/dom/Element.h"))
        .include(add_include("mozilla/ServoElementSnapshot.h"))
        .raw_line("use atomic_refcell::AtomicRefCell;")
        .raw_line("use data::ElementData;")
        .raw_line("pub use nsstring::nsStringRepr as nsString;")
        .hide_type("nsString")
        .whitelisted_vars(&[
            "NS_THEME_.*",
            "NODE_.*",
            "NS_FONT_.*",
            "NS_STYLE_.*",
            "NS_CORNER_.*",
            "NS_RADIUS_.*",
            "BORDER_COLOR_.*",
            "BORDER_STYLE_.*"
        ])
        .whitelisted_types(&[
            "AnonymousContent",
            "AudioContext",
            "CapturingContentInfo",
            "DefaultDelete",
            "DOMIntersectionObserverEntry",
            "Element",
            "FontFamilyList",
            "FontFamilyListRefCnt",
            "FontFamilyName",
            "FontFamilyType",
            "FragmentOrURL",
            "FrameRequestCallback",
            "gfxAlternateValue",
            "gfxFontFeature",
            "GridNamedArea",
            "Image",
            "ImageURL",
            "nsAttrName",
            "nsAttrValue",
            "nsBorderColors",
            "nsChangeHint",
            "nscolor",
            "nsCSSKeyword",
            "nsCSSPropertyID",
            "nsCSSRect",
            "nsCSSRect_heap",
            "nsCSSShadowArray",
            "nsCSSValue",
            "nsCSSValueFloatColor",
            "nsCSSValueGradient",
            "nsCSSValueGradientStop",
            "nsCSSValueList",
            "nsCSSValueList_heap",
            "nsCSSValuePair_heap",
            "nsCSSValuePairList",
            "nsCSSValuePairList_heap",
            "nsCSSValueTokenStream",
            "nsCSSValueTriplet_heap",
            "nsCursorImage",
            "nsFont",
            "nsIAtom",
            "nsMainThreadPtrHandle",
            "nsMainThreadPtrHolder",
            "nsMargin",
            "nsRect",
            "nsRestyleHint",
            "nsresult",
            "nsSize",
            "nsStyleBackground",
            "nsStyleBorder",
            "nsStyleColor",
            "nsStyleColumn",
            "nsStyleContent",
            "nsStyleContentData",
            "nsStyleContext",
            "nsStyleCoord",
            "nsStyleCoord",
            "nsStyleCoord",
            "nsStyleCounterData",
            "nsStyleDisplay",
            "nsStyleEffects",
            "nsStyleFilter",
            "nsStyleFont",
            "nsStyleGradient",
            "nsStyleGradientStop",
            "nsStyleImage",
            "nsStyleImageLayers",
            "nsStyleImageLayers_Layer",
            "nsStyleImageLayers_LayerType",
            "nsStyleList",
            "nsStyleMargin",
            "nsStyleOutline",
            "nsStylePadding",
            "nsStylePosition",
            "nsStyleSVG",
            "nsStyleSVGReset",
            "nsStyleTable",
            "nsStyleTableBorder",
            "nsStyleText",
            "nsStyleTextReset",
            "nsStyleUIReset",
            "nsStyleUnion",
            "nsStyleUnit",
            "nsStyleUserInterface",
            "nsStyleVariables",
            "nsStyleVisibility",
            "nsStyleXUL",
            "nsTArray",
            "nsTArrayHeader",
            "pair",
            "Position",
            "Runnable",
            "ServoAttrSnapshot",
            "ServoElementSnapshot",
            "SheetParsingMode",
            "Side",
            "StaticRefPtr",
            "StyleAnimation",
            "StyleBasicShape",
            "StyleBasicShape",
            "StyleBasicShapeType",
            "StyleClipPath",
            "StyleClipPathGeometryBox",
            "StyleTransition",
            "UniquePtr",
        ])
        .opaque_types(&[
            "atomic___base", "atomic__My_base",
            "nsAString_internal_char_traits",
            "nsAString_internal_incompatible_char_type",
            "nsACString_internal_char_traits",
            "nsACString_internal_incompatible_char_type",
            "RefPtr_Proxy", "nsAutoPtr_Proxy", "Pair_Base",
            "RefPtr_Proxy_member_function", "nsAutoPtr_Proxy_member_function",
            "nsWritingIterator_reference", "nsReadingIterator_reference",
            "Heap", "TenuredHeap", "Rooted", "WeakPtr",  // <- More template magic than what
                                                         //    we support.
            "nsTObserverArray", // <- Inherits from nsAutoTObserverArray<T, 0>
            "PLArenaPool",  // <- Bindgen bug
            "nsTHashtable",  // <- Inheriting from inner typedefs that clang
                             //    doesn't expose properly.
            "nsRefPtrHashtable", "nsDataHashtable", "nsClassHashtable",  // <- Ditto
            "nsIDocument_SelectorCache",  // <- Inherits from nsExpirationTracker<.., 4>
            "nsIPresShell_ScrollAxis",  // <- For some reason the alignment of this is 4
                                        // for clang.
            "nsPIDOMWindow",  // <- Takes the vtable from a template parameter, and we can't
                              //    generate it conditionally.
            "SupportsWeakPtr",
            "Maybe",  // <- AlignedStorage, which means templated union, which
                      //    means impossible to represent in stable rust as of
                      //    right now.
            "gfxSize",  // <- Same, union { struct { T width; T height; }; T components[2] };
            "gfxSize_Super",  // Ditto.
        ]);

    struct MappedGenericType {
        generic: bool,
        gecko: &'static str,
        servo: &'static str,
    }
    let servo_mapped_generic_types = [
        MappedGenericType {
            generic: true,
            gecko: "ServoUnsafeCell",
            servo: "::std::cell::UnsafeCell"
        },
        MappedGenericType {
            generic: true,
            gecko: "ServoCell",
            servo: "::std::cell::Cell"
        },
        MappedGenericType {
            generic: false,
            gecko: "ServoNodeData",
            servo: "AtomicRefCell<ElementData>",
        }
    ];
    for ty in servo_mapped_generic_types.iter() {
        let generic = if ty.generic { "<T>" } else { "" };
        builder = builder.hide_type(ty.gecko)
            .raw_line(format!("pub type {0}{2} = {1}{2};", ty.gecko, ty.servo, generic));
    }
    write_binding_file(builder, GECKO_STRUCTS);
}

pub fn generate() {
    if let Some(_) = *OBJDIR {
        generate_bindings();
        generate_structs();
    } else {
        // If objdir environment variable is not available, copy the
        // in-tree binding code to outdir.
        let bindings_path = Path::new(file!()).parent().unwrap().join("gecko_bindings");
        fs::copy(bindings_path.join("bindings.rs"), OUTDIR_PATH.join(GECKO_BINDINGS)).unwrap();
        fs::copy(bindings_path.join(if cfg!(debug_assertions) {
            "structs_debug.rs"
        } else {
            "structs_release.rs"
        }), OUTDIR_PATH.join(GECKO_STRUCTS)).unwrap();
    }
}
