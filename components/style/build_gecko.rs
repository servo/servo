/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod common {
    use std::env;
    use std::path::PathBuf;

    lazy_static! {
        pub static ref OUTDIR_PATH: PathBuf = PathBuf::from(env::var("OUT_DIR").unwrap()).join("gecko");
    }

    pub const STRUCTS_DEBUG_FILE: &'static str = "structs_debug.rs";
    pub const STRUCTS_RELEASE_FILE: &'static str = "structs_release.rs";
    pub const BINDINGS_FILE: &'static str = "bindings.rs";

    #[derive(Clone, Copy, PartialEq)]
    pub enum BuildType {
        Debug,
        Release,
    }

    pub fn structs_file(build_type: BuildType) -> &'static str {
        match build_type {
            BuildType::Debug => STRUCTS_DEBUG_FILE,
            BuildType::Release => STRUCTS_RELEASE_FILE
        }
    }
}

#[cfg(feature = "bindgen")]
mod bindings {
    use bindgen::{Builder, CodegenConfig};
    use bindgen::chooser::{EnumVariantCustomBehavior, EnumVariantValue, TypeChooser};
    use regex::Regex;
    use std::cmp;
    use std::collections::HashSet;
    use std::env;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::SystemTime;
    use super::common::*;

    lazy_static! {
        static ref INCLUDE_RE: Regex = Regex::new(r#"#include\s*"(.+?)""#).unwrap();
        static ref DISTDIR_PATH: PathBuf = {
            let path = PathBuf::from(env::var("MOZ_DIST").unwrap());
            if !path.is_absolute() || !path.is_dir() {
                panic!("MOZ_DIST must be an absolute directory, was: {}", path.display());
            }
            path
        };
        static ref SEARCH_PATHS: Vec<PathBuf> = vec![
            DISTDIR_PATH.join("include"),
            DISTDIR_PATH.join("include/nspr"),
        ];
        static ref ADDED_PATHS: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
        pub static ref LAST_MODIFIED: Mutex<SystemTime> =
            Mutex::new(get_modified_time(&env::current_exe().unwrap())
                       .expect("Failed to get modified time of executable"));
        static ref BINDING_DISTDIR_PATH: PathBuf = {
            let path = DISTDIR_PATH.join("rust_bindings/style");
            fs::create_dir_all(&path).expect("Fail to create bindings dir in dist");
            path
        };
    }

    fn get_modified_time(file: &Path) -> Option<SystemTime> {
        file.metadata().and_then(|m| m.modified()).ok()
    }

    fn search_include(name: &str) -> Option<PathBuf> {
        for path in SEARCH_PATHS.iter() {
            let file = path.join(name);
            if !file.is_file() {
                continue;
            }
            let modified = get_modified_time(&file).unwrap();
            let mut last_modified = LAST_MODIFIED.lock().unwrap();
            *last_modified = cmp::max(modified, *last_modified);
            return Some(file);
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
            if let Some(path) = search_include(cap.get(1).unwrap().as_str()) {
                add_headers_recursively(path, added_paths);
            }
        }
    }

    fn add_include(name: &str) -> String {
        let mut added_paths = ADDED_PATHS.lock().unwrap();
        let file = search_include(name).expect("Include not found!");
        let result = String::from(file.to_str().unwrap());
        add_headers_recursively(file, &mut *added_paths);
        result
    }

    trait BuilderExt {
        fn get_initial_builder(build_type: BuildType) -> Builder;
        fn include<T: Into<String>>(self, file: T) -> Builder;
        fn zero_size_type(self, ty: &str, structs_list: &[&str]) -> Builder;
        fn borrowed_type(self, ty: &str) -> Builder;
        fn mutable_borrowed_type(self, ty: &str) -> Builder;
    }

    impl BuilderExt for Builder {
        fn get_initial_builder(build_type: BuildType) -> Builder {
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
            builder = builder.include(add_include("mozilla-config.h"));

            if build_type == BuildType::Debug {
                builder = builder.clang_arg("-DDEBUG=1").clang_arg("-DJS_DEBUG=1");
            }
            if cfg!(target_family = "unix") {
                builder = builder.clang_arg("-DOS_POSIX=1");
            }
            if cfg!(target_os = "linux") {
                builder = builder.clang_arg("-DOS_LINUX=1");
            } else if cfg!(target_os = "solaris") {
                builder = builder.clang_arg("-DOS_SOLARIS=1");
            } else if cfg!(target_os = "dragonfly") {
                builder = builder.clang_arg("-DOS_BSD=1").clang_arg("-DOS_DRAGONFLY=1");
            } else if cfg!(target_os = "freebsd") {
                builder = builder.clang_arg("-DOS_BSD=1").clang_arg("-DOS_FREEBSD=1");
            } else if cfg!(target_os = "netbsd") {
                builder = builder.clang_arg("-DOS_BSD=1").clang_arg("-DOS_NETBSD=1");
            } else if cfg!(target_os = "openbsd") {
                builder = builder.clang_arg("-DOS_BSD=1").clang_arg("-DOS_OPENBSD=1");
            } else if cfg!(target_os = "macos") {
                builder = builder.clang_arg("-DOS_MACOSX=1")
                    .clang_arg("-stdlib=libc++")
                    // To disable the fixup bindgen applies which adds search
                    // paths from clang command line in order to avoid potential
                    // conflict with -stdlib=libc++.
                    .clang_arg("--target=x86_64-apple-darwin");
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
        fn zero_size_type(self, ty: &str, structs_list: &[&str]) -> Builder {
            if !structs_list.contains(&ty) {
                self.hide_type(ty)
                    .raw_line(format!("enum {}Void {{ }}", ty))
                    .raw_line(format!("pub struct {0}({0}Void);", ty))
            } else {
                self
            }
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
    }

    struct Fixup {
        pat: String,
        rep: String
    }

    fn write_binding_file(builder: Builder, file: &str, fixups: &[Fixup]) {
        let out_file = OUTDIR_PATH.join(file);
        if let Some(modified) = get_modified_time(&out_file) {
            // Don't generate the file if nothing it depends on was modified.
            let last_modified = LAST_MODIFIED.lock().unwrap();
            if *last_modified <= modified {
                return;
            }
        }
        let mut result = builder.generate().expect("Unable to generate bindings").to_string();
        for fixup in fixups.iter() {
            result = Regex::new(&format!(r"\b{}\b", fixup.pat)).unwrap().replace_all(&result, fixup.rep.as_str())
                .into_owned().into();
        }
        let bytes = result.into_bytes();
        File::create(&out_file).unwrap().write_all(&bytes).expect("Unable to write output");
        File::create(&BINDING_DISTDIR_PATH.join(file)).unwrap()
            .write_all(&bytes).expect("Unable to write output to binding dist");
    }

    fn get_arc_types() -> Vec<String> {
        // Read the file
        let mut list_file = File::open(DISTDIR_PATH.join("include/mozilla/ServoArcTypeList.h"))
            .expect("Unable to open ServoArcTypeList.h");
        let mut content = String::new();
        list_file.read_to_string(&mut content).expect("Fail to read ServoArcTypeList.h");
        // Remove comments
        let block_comment_re = Regex::new(r#"(?s)/\*.*?\*/"#).unwrap();
        let content = block_comment_re.replace_all(&content, "");
        // Extract the list
        let re = Regex::new(r#"^SERVO_ARC_TYPE\(\w+,\s*(\w+)\)$"#).unwrap();
        content.lines().map(|line| line.trim()).filter(|line| !line.is_empty())
            .map(|line| re.captures(&line)
                 .expect(&format!("Unrecognized line in ServoArcTypeList.h: '{}'", line))
                 .get(1).unwrap().as_str().to_string())
            .collect()
    }

    #[derive(Debug)]
    struct Callbacks;
    impl TypeChooser for Callbacks {
        fn enum_variant_behavior(&self,
                                 enum_name: Option<&str>,
                                 variant_name: &str,
                                 _variant_value: EnumVariantValue)
                                 -> Option<EnumVariantCustomBehavior> {
            if enum_name.map_or(false, |n| n == "nsCSSPropertyID") &&
               variant_name.starts_with("eCSSProperty_COUNT") {
                Some(EnumVariantCustomBehavior::Constify)
            } else {
                None
            }
        }
    }

    pub fn generate_structs(build_type: BuildType) {
        let mut builder = Builder::get_initial_builder(build_type)
            .enable_cxx_namespaces()
            .with_codegen_config(CodegenConfig {
                types: true,
                vars: true,
                ..CodegenConfig::nothing()
            })
            .include(add_include("nsCSSPseudoClasses.h"))   // servo/rust-bindgen#599
            .header(add_include("nsStyleStruct.h"))
            .include(add_include("mozilla/ServoPropPrefList.h"))
            .header(add_include("mozilla/StyleAnimationValue.h"))
            .include(add_include("gfxFontConstants.h"))
            .include(add_include("nsThemeConstants.h"))
            .include(add_include("mozilla/dom/AnimationEffectReadOnlyBinding.h"))
            .include(add_include("mozilla/AnimationPropertySegment.h"))
            .include(add_include("mozilla/ComputedTiming.h"))
            .include(add_include("mozilla/ComputedTimingFunction.h"))
            .include(add_include("mozilla/Keyframe.h"))
            .include(add_include("mozilla/ServoElementSnapshot.h"))
            .include(add_include("mozilla/dom/Element.h"))
            .include(add_include("mozilla/dom/NameSpaceConstants.h"))
            .include(add_include("mozilla/LookAndFeel.h"))
            .include(add_include("mozilla/ServoBindings.h"))
            .include(add_include("nsCSSFontFaceRule.h"))
            .include(add_include("nsMediaFeatures.h"))
            .include(add_include("nsMediaList.h"))
            // FIXME(emilio): Incrementally remove these "pub use"s. Probably
            // mozilla::css and mozilla::dom are easier.
            .raw_line("pub use self::root::*;")
            .raw_line("pub use self::root::mozilla::*;")
            .raw_line("pub use self::root::mozilla::css::*;")
            .raw_line("pub use self::root::mozilla::dom::*;")
            .raw_line("use atomic_refcell::AtomicRefCell;")
            .raw_line("use data::ElementData;")
            .hide_type("nsString")
            .bitfield_enum("nsChangeHint")
            .bitfield_enum("nsRestyleHint")
            .constified_enum("UpdateAnimationsTasks")
            .type_chooser(Box::new(Callbacks));
        let whitelist_vars = [
            "NS_THEME_.*",
            "NODE_.*",
            "NS_FONT_.*",
            "NS_STYLE_.*",
            "NS_MATHML_.*",
            "NS_RADIUS_.*",
            "BORDER_COLOR_.*",
            "BORDER_STYLE_.*",
            "mozilla::SERVO_PREF_.*",
            "kNameSpaceID_.*",
            "kGenericFont_.*",
            "kPresContext_.*",
        ];
        let whitelist = [
            "RawGecko.*",
            "mozilla::AnimationPropertySegment",
            "mozilla::ComputedTiming",
            "mozilla::ComputedTimingFunction",
            "mozilla::ComputedTimingFunction::BeforeFlag",
            "mozilla::ServoStyleSheet",
            "mozilla::ServoElementSnapshot.*",
            "mozilla::CSSPseudoClassType",
            "mozilla::css::SheetParsingMode",
            "mozilla::HalfCorner",
            "mozilla::PropertyStyleAnimationValuePair",
            "mozilla::TraversalRestyleBehavior",
            "mozilla::TraversalRootBehavior",
            "mozilla::StyleShapeRadius",
            "mozilla::StyleGrid.*",
            "mozilla::UpdateAnimationsTasks",
            "mozilla::LookAndFeel",
            ".*ThreadSafe.*Holder",
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
            "FontSizePrefs",
            "FragmentOrURL",
            "FrameRequestCallback",
            "GeckoParserExtraData",
            "GeckoFontMetrics",
            "gfxAlternateValue",
            "gfxFontFeature",
            "gfxFontVariation",
            "GridNamedArea",
            "HalfCorner",
            "Image",
            "ImageURL",
            "Keyframe",
            "nsAttrName",
            "nsAttrValue",
            "nsBorderColors",
            "nscolor",
            "nsChangeHint",
            "nsCSSFontFaceRule",
            "nsCSSKeyword",
            "nsCSSPropertyID",
            "nsCSSProps",
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
            "nsMediaExpression",
            "nsMediaFeature",
            "nsMediaFeatures",
            "nsMediaList",
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
            "nsStyleCounterData",
            "nsStyleDisplay",
            "nsStyleEffects",
            "nsStyleFilter",
            "nsStyleFont",
            "nsStyleGradient",
            "nsStyleGradientStop",
            "nsStyleGridTemplate",
            "nsStyleImage",
            "nsStyleImageLayers",
            "nsStyleList",
            "nsStyleMargin",
            "nsStyleOutline",
            "nsStylePadding",
            "nsStylePosition",
            "nsStyleSVG",
            "nsStyleSVGPaint",
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
            "Position",
            "PropertyValuePair",
            "Runnable",
            "ServoAttrSnapshot",
            "ServoBundledURI",
            "ServoElementSnapshot",
            "SheetParsingMode",
            "StaticRefPtr",
            "StyleAnimation",
            "StyleBasicShape",
            "StyleBasicShapeType",
            "StyleGeometryBox",
            "StyleShapeSource",
            "StyleTransition",
            "mozilla::UniquePtr",
            "mozilla::DefaultDelete",
            "mozilla::Side",
            "mozilla::binding_danger::AssertAndSuppressCleanupPolicy",
            "mozilla::LengthParsingMode",
            "mozilla::InheritTarget",
        ];
        let opaque_types = [
            "std::pair__PCCP",
            "std::namespace::atomic___base", "std::atomic__My_base",
            "std::atomic",
            "std::atomic___base",
            "mozilla::gfx::.*",
            "FallibleTArray",
            "mozilla::dom::Sequence",
            "mozilla::dom::Optional",
            "mozilla::dom::Nullable",
            "RefPtr_Proxy",
            "RefPtr_Proxy_member_function",
            "nsAutoPtr_Proxy",
            "nsAutoPtr_Proxy_member_function",
            "mozilla::detail::PointerType",
            "mozilla::Pair_Base",
            "mozilla::SupportsWeakPtr",
            "SupportsWeakPtr",
            "mozilla::detail::WeakReference",
            "mozilla::WeakPtr",
            "nsWritingIterator_reference", "nsReadingIterator_reference",
            "nsTObserverArray",  // <- Inherits from nsAutoTObserverArray<T, 0>
            "nsTHashtable",  // <- Inheriting from inner typedefs that clang
                             //    doesn't expose properly.
            "nsRefPtrHashtable", "nsDataHashtable", "nsClassHashtable",  // <- Ditto
            "nsIDocument_SelectorCache",  // <- Inherits from nsExpirationTracker<.., 4>
            "nsIPresShell_ScrollAxis",  // <- For some reason the alignment of this is 4
                                        // for clang.
            "nsPIDOMWindow",  // <- Takes the vtable from a template parameter, and we can't
                              //    generate it conditionally.
            "JS::Rooted",
            "mozilla::Maybe",
            "gfxSize",  // <- union { struct { T width; T height; }; T components[2] };
            "gfxSize_Super",  // Ditto.
            "mozilla::ErrorResult",  // Causes JSWhyMagic to be included & handled incorrectly.
            "mozilla::StyleAnimationValue",
            "StyleAnimationValue", // pulls in a whole bunch of stuff we don't need in the bindings
        ];
        let blacklist = [
            ".*_char_traits",
            ".*_incompatible_char_type",
        ];

        struct MappedGenericType {
            generic: bool,
            gecko: &'static str,
            servo: &'static str,
        }
        let servo_mapped_generic_types = [
            MappedGenericType {
                generic: true,
                gecko: "mozilla::ServoUnsafeCell",
                servo: "::std::cell::UnsafeCell"
            },
            MappedGenericType {
                generic: true,
                gecko: "mozilla::ServoCell",
                servo: "::std::cell::Cell"
            },
            MappedGenericType {
                generic: false,
                gecko: "ServoNodeData",
                servo: "AtomicRefCell<ElementData>",
            }
        ];
        let mut fixups = vec![
            Fixup {
                pat: "root::nsString".into(),
                rep: "::nsstring::nsStringRepr".into()
            },
        ];
        for &var in whitelist_vars.iter() {
            builder = builder.whitelisted_var(var);
        }
        for &ty in whitelist.iter() {
            builder = builder.whitelisted_type(ty);
        }
        for &ty in opaque_types.iter() {
            builder = builder.opaque_type(ty);
        }
        for &ty in blacklist.iter() {
            builder = builder.hide_type(ty);
        }
        for ty in servo_mapped_generic_types.iter() {
            let gecko_name = ty.gecko.rsplit("::").next().unwrap();
            builder = builder.hide_type(ty.gecko)
                .raw_line(format!("pub type {0}{2} = {1}{2};", gecko_name, ty.servo,
                                  if ty.generic { "<T>" } else { "" }));
            fixups.push(Fixup {
                pat: format!("root::{}", ty.gecko),
                rep: format!("::gecko_bindings::structs::{}", gecko_name)
            });
        }
        write_binding_file(builder, structs_file(build_type), &fixups);
    }

    pub fn setup_logging() {
        use log;

        struct BuildLogger {
            file: Option<Mutex<fs::File>>,
            filter: String,
        }

        impl log::Log for BuildLogger {
            fn enabled(&self, meta: &log::LogMetadata) -> bool {
                self.file.is_some() && meta.target().contains(&self.filter)
            }

            fn log(&self, record: &log::LogRecord) {
                if !self.enabled(record.metadata()) {
                    return;
                }

                let mut file = self.file.as_ref().unwrap().lock().unwrap();
                let _ =
                    writeln!(file, "{} - {} - {} @ {}:{}",
                             record.level(),
                             record.target(),
                             record.args(),
                             record.location().file(),
                             record.location().line());
            }
        }

        log::set_logger(|log_level| {
            log_level.set(log::LogLevelFilter::Debug);
            Box::new(BuildLogger {
                file: env::var("STYLO_BUILD_LOG").ok().and_then(|path| {
                    fs::File::create(path).ok().map(Mutex::new)
                }),
                filter: env::var("STYLO_BUILD_FILTER").ok()
                    .unwrap_or_else(|| "bindgen".to_owned()),
            })
        })
        .expect("Failed to set logger.");
    }

    pub fn generate_bindings() {
        let mut builder = Builder::get_initial_builder(BuildType::Release)
            .disable_name_namespacing()
            .with_codegen_config(CodegenConfig {
                functions: true,
                ..CodegenConfig::nothing()
            })
            .header(add_include("mozilla/ServoBindings.h"))
            .hide_type("nsACString_internal")
            .hide_type("nsAString_internal")
            .raw_line("pub use nsstring::{nsACString, nsAString, nsString};")
            .raw_line("type nsACString_internal = nsACString;")
            .raw_line("type nsAString_internal = nsAString;")
            .whitelisted_function("Servo_.*")
            .whitelisted_function("Gecko_.*");
        let structs_types = [
            "mozilla::css::URLValue",
            "mozilla::Side",
            "RawGeckoAnimationPropertySegment",
            "RawGeckoComputedTiming",
            "RawGeckoDocument",
            "RawGeckoElement",
            "RawGeckoKeyframeList",
            "RawGeckoComputedKeyframeValuesList",
            "RawGeckoFontFaceRuleList",
            "RawGeckoNode",
            "RawGeckoAnimationValueList",
            "RawServoAnimationValue",
            "RawServoDeclarationBlock",
            "RawServoStyleRule",
            "RawGeckoPresContext",
            "RawGeckoPresContextOwned",
            "RawGeckoStyleAnimationList",
            "RawGeckoServoStyleRuleList",
            "RawGeckoURLExtraData",
            "RefPtr",
            "CSSPseudoClassType",
            "TraversalRestyleBehavior",
            "TraversalRootBehavior",
            "ComputedTimingFunction_BeforeFlag",
            "FontFamilyList",
            "FontFamilyType",
            "FontSizePrefs",
            "GeckoFontMetrics",
            "Keyframe",
            "ServoBundledURI",
            "ServoElementSnapshot",
            "SheetParsingMode",
            "StyleBasicShape",
            "StyleBasicShapeType",
            "StyleShapeSource",
            "StyleTransition",
            "nsCSSFontFaceRule",
            "nsCSSKeyword",
            "nsCSSPropertyID",
            "nsCSSShadowArray",
            "nsCSSUnit",
            "nsCSSValue",
            "nsCSSValueSharedList",
            "nsChangeHint",
            "nsCursorImage",
            "nsFont",
            "nsIAtom",
            "nsMediaFeature",
            "nsRestyleHint",
            "nsStyleBackground",
            "nsStyleBorder",
            "nsStyleColor",
            "nsStyleColumn",
            "nsStyleContent",
            "nsStyleContentData",
            "nsStyleContentType",
            "nsStyleContext",
            "nsStyleCoord",
            "nsStyleCoord_Calc",
            "nsStyleCoord_CalcValue",
            "nsStyleDisplay",
            "nsStyleEffects",
            "nsStyleFilter",
            "nsStyleFont",
            "nsStyleGradient",
            "nsStyleGradientStop",
            "nsStyleGridTemplate",
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
            "nsStyleSVGPaint",
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
            "nsTimingFunction",
            "nscolor",
            "nscoord",
            "nsresult",
            "Loader",
            "ServoStyleSheet",
            "EffectCompositor_CascadeLevel",
            "UpdateAnimationsTasks",
            "LengthParsingMode",
            "InheritTarget",
        ];
        struct ArrayType {
            cpp_type: &'static str,
            rust_type: &'static str
        }
        let array_types = [
            ArrayType { cpp_type: "uintptr_t", rust_type: "usize" },
        ];
        struct ServoOwnedType {
            name: &'static str,
            opaque: bool,
        }
        let servo_owned_types = [
            ServoOwnedType { name: "RawServoStyleSet", opaque: true },
            ServoOwnedType { name: "StyleChildrenIterator", opaque: true },
            ServoOwnedType { name: "ServoElementSnapshot", opaque: false },
            ServoOwnedType { name: "RawServoAnimationValueMap", opaque: true },
        ];
        let servo_immutable_borrow_types = [
            "RawGeckoNode",
            "RawGeckoElement",
            "RawGeckoDocument",
            "RawServoDeclarationBlockStrong",
            "RawGeckoPresContext",
            "RawGeckoStyleAnimationList",
        ];
        let servo_borrow_types = [
            "nsCSSValue",
            "nsTimingFunction",
            "RawGeckoAnimationPropertySegment",
            "RawGeckoAnimationValueList",
            "RawGeckoComputedTiming",
            "RawGeckoKeyframeList",
            "RawGeckoComputedKeyframeValuesList",
            "RawGeckoFontFaceRuleList",
            "RawGeckoServoStyleRuleList",
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
        for &ArrayType { cpp_type, rust_type } in array_types.iter() {
            builder = builder.hide_type(format!("nsTArrayBorrowed_{}", cpp_type))
                .raw_line(format!("pub type nsTArrayBorrowed_{}<'a> = &'a mut ::gecko_bindings::structs::nsTArray<{}>;",
                                  cpp_type, rust_type))
        }
        for ty in get_arc_types().iter() {
            builder = builder
                .hide_type(format!("{}Strong", ty))
                .raw_line(format!("pub type {0}Strong = ::gecko_bindings::sugar::ownership::Strong<{0}>;", ty))
                .borrowed_type(ty)
                .zero_size_type(ty, &structs_types);
        }
        for &ServoOwnedType { name, opaque } in servo_owned_types.iter() {
            builder = builder
                .hide_type(format!("{}Owned", name))
                .raw_line(format!("pub type {0}Owned = ::gecko_bindings::sugar::ownership::Owned<{0}>;", name))
                .hide_type(format!("{}OwnedOrNull", name))
                .raw_line(format!("pub type {0}OwnedOrNull = ::gecko_bindings::sugar::ownership::OwnedOrNull<{0}>;",
                                  name))
                .mutable_borrowed_type(name);
            if opaque {
                builder = builder.zero_size_type(name, &structs_types);
            }
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
        write_binding_file(builder, BINDINGS_FILE, &Vec::new());
    }
}

#[cfg(not(feature = "bindgen"))]
mod bindings {
    use std::fs;
    use std::path::{Path, PathBuf};
    use super::common::*;

    lazy_static! {
        static ref BINDINGS_PATH: PathBuf = Path::new(file!()).parent().unwrap().join("gecko_bindings");
    }

    pub fn setup_logging() {}

    pub fn generate_structs(build_type: BuildType) {
        let file = structs_file(build_type);
        let source = BINDINGS_PATH.join(file);
        println!("cargo:rerun-if-changed={}", source.display());
        fs::copy(source, OUTDIR_PATH.join(file)).unwrap();
    }

    pub fn generate_bindings() {
        let source = BINDINGS_PATH.join(BINDINGS_FILE);
        println!("cargo:rerun-if-changed={}", source.display());
        fs::copy(source, OUTDIR_PATH.join(BINDINGS_FILE)).unwrap();
    }
}

pub fn generate() {
    use self::common::*;
    use std::{env, fs, thread};
    println!("cargo:rerun-if-changed=build_gecko.rs");
    fs::create_dir_all(&*OUTDIR_PATH).unwrap();
    bindings::setup_logging();
    if env::var("STYLO_BUILD_LOG").is_ok() {
        bindings::generate_structs(BuildType::Debug);
        bindings::generate_structs(BuildType::Release);
        bindings::generate_bindings();
    } else {
        let threads = vec![
            thread::spawn(|| bindings::generate_structs(BuildType::Debug)),
            thread::spawn(|| bindings::generate_structs(BuildType::Release)),
            thread::spawn(|| bindings::generate_bindings()),
        ];
        for t in threads.into_iter() {
            t.join().unwrap();
        }
    }
}
