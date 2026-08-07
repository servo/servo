/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use indexmap::IndexMap;
use js::context::JSContext;
use script_bindings::error::Fallible;
use serde_json::{Map as JsonMap, Value as JsonValue};
use servo_url::ServoUrl;

use crate::dom::bindings::error::{Error, report_pending_exception, throw_dom_exception};
use crate::dom::console::Console;
use crate::dom::globalscope::GlobalScope;
use crate::realms::enter_auto_realm;

type ModuleIntegrityMap = IndexMap<ServoUrl, String>;
pub(crate) type ModuleSpecifierMap = IndexMap<String, Option<ServoUrl>>;

/// <https://html.spec.whatwg.org/multipage/#import-map-processing-model>
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct ImportMap {
    #[no_trace]
    pub(crate) imports: ModuleSpecifierMap,
    #[no_trace]
    pub(crate) scopes: IndexMap<ServoUrl, ModuleSpecifierMap>,
    #[no_trace]
    integrity: ModuleIntegrityMap,
}

impl ImportMap {
    /// <https://html.spec.whatwg.org/multipage/#resolving-a-module-integrity-metadata>
    pub(crate) fn resolve_a_module_integrity_metadata(&self, url: &ServoUrl) -> String {
        // Step 1. Let map be settingsObject's global object's import map.

        // Step 2. If map's integrity[url] does not exist, then return the empty string.
        // Step 3. Return map's integrity[url].
        self.integrity.get(url).cloned().unwrap_or_default()
    }
}

/// <https://html.spec.whatwg.org/multipage/#register-an-import-map>
pub(crate) fn register_import_map(
    cx: &mut JSContext,
    global: &GlobalScope,
    result: Fallible<ImportMap>,
) {
    match result {
        Ok(new_import_map) => {
            // Step 2. Merge existing and new import maps, given global and result's import map.
            merge_existing_and_new_import_maps(cx, global, new_import_map);
        },
        Err(exception) => {
            let mut realm = enter_auto_realm(cx, global);
            let cx = &mut realm.current_realm();

            // Step 1. If result's error to rethrow is not null, then report
            // an exception given by result's error to rethrow for global and return.
            throw_dom_exception(cx, global, exception);
            report_pending_exception(cx);
        },
    }
}

/// <https://html.spec.whatwg.org/multipage/#merge-existing-and-new-import-maps>
fn merge_existing_and_new_import_maps(
    cx: &mut JSContext,
    global: &GlobalScope,
    new_import_map: ImportMap,
) {
    // Step 1. Let newImportMapScopes be a deep copy of newImportMap's scopes.
    let new_import_map_scopes = new_import_map.scopes;

    // Step 2. Let oldImportMap be global's import map.
    let mut old_import_map = global.import_map_mut();

    // Step 3. Let newImportMapImports be a deep copy of newImportMap's imports.
    let mut new_import_map_imports = new_import_map.imports;

    let resolved_module_set = global.resolved_module_set();
    // Step 4. For each scopePrefix → scopeImports of newImportMapScopes:
    for (scope_prefix, mut scope_imports) in new_import_map_scopes {
        // Step 4.1. For each record of global's resolved module set:
        for record in resolved_module_set.iter() {
            // If scopePrefix is record's serialized base URL, or if scopePrefix ends with
            // U+002F (/) and scopePrefix is a code unit prefix of record's serialized base URL, then:
            let prefix = scope_prefix.as_str();
            if prefix == record.base_url ||
                (record.base_url.starts_with(prefix) && prefix.ends_with('\u{002f}'))
            {
                // For each specifierKey → resolutionResult of scopeImports:
                scope_imports.retain(|key, val| {
                    // If specifierKey is record's specifier, or if all of the following conditions are true:
                    // specifierKey ends with U+002F (/);
                    // specifierKey is a code unit prefix of record's specifier;
                    // either record's specifier as a URL is null or is special,
                    if *key == record.specifier ||
                        (key.ends_with('\u{002f}') &&
                            record.specifier.starts_with(key) &&
                            (record.specifier_url.is_none() ||
                                record
                                    .specifier_url
                                    .as_ref()
                                    .is_some_and(|u| u.is_special_scheme())))
                    {
                        // The user agent may report a warning to the console indicating the ignored rule.
                        // They may choose to avoid reporting if the rule is identical to an existing one.
                        Console::internal_warn(
                            cx,
                            global,
                            format!("Ignored rule: {key} -> {val:?}."),
                        );
                        // Remove scopeImports[specifierKey].
                        false
                    } else {
                        true
                    }
                })
            }
        }

        // Step 4.2 If scopePrefix exists in oldImportMap's scopes
        if old_import_map.scopes.contains_key(&scope_prefix) {
            // set oldImportMap's scopes[scopePrefix] to the result of
            // merging module specifier maps, given scopeImports and oldImportMap's scopes[scopePrefix].
            let merged_module_specifier_map = merge_module_specifier_maps(
                cx,
                global,
                scope_imports,
                &old_import_map.scopes[&scope_prefix],
            );
            old_import_map
                .scopes
                .insert(scope_prefix, merged_module_specifier_map);
        } else {
            // Step 4.3 Otherwise, set oldImportMap's scopes[scopePrefix] to scopeImports.
            old_import_map.scopes.insert(scope_prefix, scope_imports);
        }
    }

    // Step 5. For each url → integrity of newImportMap's integrity:
    for (url, integrity) in &new_import_map.integrity {
        // Step 5.1 If url exists in oldImportMap's integrity, then:
        if old_import_map.integrity.contains_key(url) {
            // Step 5.1.1 The user agent may report a warning to the console indicating the ignored rule.
            // They may choose to avoid reporting if the rule is identical to an existing one.
            Console::internal_warn(cx, global, format!("Ignored rule: {url} -> {integrity}."));
            // Step 5.1.2 Continue.
            continue;
        }

        // Step 5.2 Set oldImportMap's integrity[url] to integrity.
        old_import_map
            .integrity
            .insert(url.clone(), integrity.clone());
    }

    // Step 6. For each record of global's resolved module set:
    for record in resolved_module_set.iter() {
        // For each specifier → url of newImportMapImports:
        new_import_map_imports.retain(|specifier, val| {
            // If specifier starts with record's specifier, then:
            //
            // Note: Spec is wrong, we need to check if record's specifier starts with specifier
            // See: https://github.com/whatwg/html/issues/11875
            if record.specifier.starts_with(specifier) {
                // The user agent may report a warning to the console indicating the ignored rule.
                // They may choose to avoid reporting if the rule is identical to an existing one.
                Console::internal_warn(
                    cx,
                    global,
                    format!("Ignored rule: {specifier} -> {val:?}."),
                );
                // Remove newImportMapImports[specifier].
                false
            } else {
                true
            }
        });
    }

    // Step 7. Set oldImportMap's imports to the result of merge module specifier maps,
    // given newImportMapImports and oldImportMap's imports.
    let merged_module_specifier_map =
        merge_module_specifier_maps(cx, global, new_import_map_imports, &old_import_map.imports);
    old_import_map.imports = merged_module_specifier_map;

    // https://html.spec.whatwg.org/multipage/#the-resolution-algorithm
    // Sort scopes to ensure entries are visited from most-specific to least-specific.
    old_import_map
        .scopes
        .sort_by(|a_key, _, b_key, _| b_key.cmp(a_key));
}

/// <https://html.spec.whatwg.org/multipage/#merge-module-specifier-maps>
fn merge_module_specifier_maps(
    cx: &mut JSContext,
    global: &GlobalScope,
    new_map: ModuleSpecifierMap,
    old_map: &ModuleSpecifierMap,
) -> ModuleSpecifierMap {
    // Step 1. Let mergedMap be a deep copy of oldMap.
    let mut merged_map = old_map.clone();

    // Step 2. For each specifier → url of newMap:
    for (specifier, url) in new_map {
        // Step 2.1 If specifier exists in oldMap, then:
        if old_map.contains_key(&specifier) {
            // Step 2.1.1 The user agent may report a warning to the console indicating the ignored rule.
            // They may choose to avoid reporting if the rule is identical to an existing one.
            Console::internal_warn(cx, global, format!("Ignored rule: {specifier} -> {url:?}."));

            // Step 2.1.2 Continue.
            continue;
        }

        // Step 2.2 Set mergedMap[specifier] to url.
        merged_map.insert(specifier, url);
    }

    merged_map
}

/// <https://html.spec.whatwg.org/multipage/#parse-an-import-map-string>
pub(crate) fn parse_an_import_map_string(
    cx: &mut JSContext,
    global: &GlobalScope,
    input: &str,
    base_url: ServoUrl,
) -> Fallible<ImportMap> {
    // Step 1. Let parsed be the result of parsing a JSON string to an Infra value given input.
    let parsed: JsonValue = serde_json::from_str(input)
        .map_err(|_| Error::Type(c"The value needs to be a JSON object.".to_owned()))?;
    // Step 2. If parsed is not an ordered map, then throw a TypeError indicating that the
    // top-level value needs to be a JSON object.
    let JsonValue::Object(mut parsed) = parsed else {
        return Err(Error::Type(
            c"The top-level value needs to be a JSON object.".to_owned(),
        ));
    };

    // Step 3. Let sortedAndNormalizedImports be an empty ordered map.
    let mut sorted_and_normalized_imports = ModuleSpecifierMap::new();
    // Step 4. If parsed["imports"] exists, then:
    if let Some(imports) = parsed.get("imports") {
        // Step 4.1 If parsed["imports"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "imports" top-level key needs to be a JSON object.
        let JsonValue::Object(imports) = imports else {
            return Err(Error::Type(
                c"The \"imports\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 4.2 Set sortedAndNormalizedImports to the result of sorting and
        // normalizing a module specifier map given parsed["imports"] and baseURL.
        sorted_and_normalized_imports =
            sort_and_normalize_module_specifier_map(cx, global, imports, &base_url);
    }

    // Step 5. Let sortedAndNormalizedScopes be an empty ordered map.
    let mut sorted_and_normalized_scopes: IndexMap<ServoUrl, ModuleSpecifierMap> = IndexMap::new();
    // Step 6. If parsed["scopes"] exists, then:
    if let Some(scopes) = parsed.get("scopes") {
        // Step 6.1 If parsed["scopes"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "scopes" top-level key needs to be a JSON object.
        let JsonValue::Object(scopes) = scopes else {
            return Err(Error::Type(
                c"The \"scopes\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 6.2 Set sortedAndNormalizedScopes to the result of sorting and
        // normalizing scopes given parsed["scopes"] and baseURL.
        sorted_and_normalized_scopes = sort_and_normalize_scopes(cx, global, scopes, &base_url)?;
    }

    // Step 7. Let normalizedIntegrity be an empty ordered map.
    let mut normalized_integrity = ModuleIntegrityMap::new();
    // Step 8. If parsed["integrity"] exists, then:
    if let Some(integrity) = parsed.get("integrity") {
        // Step 8.1 If parsed["integrity"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "integrity" top-level key needs to be a JSON object.
        let JsonValue::Object(integrity) = integrity else {
            return Err(Error::Type(
                c"The \"integrity\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 8.2 Set normalizedIntegrity to the result of normalizing
        // a module integrity map given parsed["integrity"] and baseURL.
        normalized_integrity = normalize_module_integrity_map(cx, global, integrity, &base_url);
    }

    // Step 9. If parsed's keys contains any items besides "imports", "scopes", or "integrity",
    // then the user agent should report a warning to the console indicating that an invalid
    // top-level key was present in the import map.
    parsed.retain(|k, _| !matches!(k.as_str(), "imports" | "scopes" | "integrity"));
    if !parsed.is_empty() {
        Console::internal_warn(
            cx,
            global,
            "Invalid top-level key was present in the import map.
                Only \"imports\", \"scopes\", and \"integrity\" are allowed."
                .to_string(),
        );
    }

    // Step 10. Return an import map
    Ok(ImportMap {
        imports: sorted_and_normalized_imports,
        scopes: sorted_and_normalized_scopes,
        integrity: normalized_integrity,
    })
}

/// <https://html.spec.whatwg.org/multipage/#sorting-and-normalizing-a-module-specifier-map>
fn sort_and_normalize_module_specifier_map(
    cx: &mut JSContext,
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
) -> ModuleSpecifierMap {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized = ModuleSpecifierMap::new();

    // Step 2. For each specifier_key -> value in originalMap
    for (specifier_key, value) in original_map {
        // Step 2.1 Let normalized_specifier_key be the result of
        // normalizing a specifier key given specifier_key and base_url.
        let Some(normalized_specifier_key) =
            normalize_specifier_key(cx, global, specifier_key, base_url)
        else {
            // Step 2.2 If normalized_specifier_key is null, then continue.
            continue;
        };

        // Step 2.3 If value is not a string, then:
        let JsonValue::String(value) = value else {
            // Step 2.3.1 The user agent may report a warning to the console
            // indicating that addresses need to be strings.
            Console::internal_warn(cx, global, "Addresses need to be strings.".to_string());

            // Step 2.3.2 Set normalized[normalized_specifier_key] to null.
            normalized.insert(normalized_specifier_key, None);
            // Step 2.3.3 Continue.
            continue;
        };

        // Step 2.4. Let address_url be the result of resolving a URL-like module specifier given value and baseURL.
        let Some(address_url) = resolve_url_like_module_specifier(value.as_str(), base_url) else {
            // Step 2.5 If address_url is null, then:
            // Step 2.5.1. The user agent may report a warning to the console
            // indicating that the address was invalid.
            Console::internal_warn(
                cx,
                global,
                format!("Value failed to resolve to module specifier: {value}"),
            );

            // Step 2.5.2 Set normalized[normalized_specifier_key] to null.
            normalized.insert(normalized_specifier_key, None);
            // Step 2.5.3 Continue.
            continue;
        };

        // Step 2.6 If specifier_key ends with U+002F (/), and the serialization of
        // address_url does not end with U+002F (/), then:
        if specifier_key.ends_with('\u{002f}') && !address_url.as_str().ends_with('\u{002f}') {
            // step 2.6.1. The user agent may report a warning to the console
            // indicating that an invalid address was given for the specifier key specifierKey;
            // since specifierKey ends with a slash, the address needs to as well.
            Console::internal_warn(
                cx,
                global,
                format!(
                    "Invalid address for specifier key '{specifier_key}': {address_url}.
                    Since specifierKey ends with a slash, the address needs to as well."
                ),
            );

            // Step 2.6.2 Set normalized[normalized_specifier_key] to null.
            normalized.insert(normalized_specifier_key, None);
            // Step 2.6.3 Continue.
            continue;
        }

        // Step 2.7 Set normalized[normalized_specifier_key] to address_url.
        normalized.insert(normalized_specifier_key, Some(address_url));
    }

    // Step 3. Return the result of sorting in descending order normalized
    // with an entry a being less than an entry b if a's key is code unit less than b's key.
    normalized.sort_by(|a_key, _, b_key, _| b_key.cmp(a_key));
    normalized
}

/// <https://html.spec.whatwg.org/multipage/#sorting-and-normalizing-scopes>
fn sort_and_normalize_scopes(
    cx: &mut JSContext,
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
) -> Fallible<IndexMap<ServoUrl, ModuleSpecifierMap>> {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized: IndexMap<ServoUrl, ModuleSpecifierMap> = IndexMap::new();

    // Step 2. For each scopePrefix → potentialSpecifierMap of originalMap:
    for (scope_prefix, potential_specifier_map) in original_map {
        // Step 2.1 If potentialSpecifierMap is not an ordered map, then throw a TypeError indicating
        // that the value of the scope with prefix scopePrefix needs to be a JSON object.
        let JsonValue::Object(potential_specifier_map) = potential_specifier_map else {
            return Err(Error::Type(
                c"The value of the scope with prefix scopePrefix needs to be a JSON object."
                    .to_owned(),
            ));
        };

        // Step 2.2 Let scopePrefixURL be the result of URL parsing scopePrefix with baseURL.
        let Ok(scope_prefix_url) = ServoUrl::parse_with_base(Some(base_url), scope_prefix) else {
            // Step 2.3 If scopePrefixURL is failure, then:
            // Step 2.3.1 The user agent may report a warning
            // to the console that the scope prefix URL was not parseable.
            Console::internal_warn(
                cx,
                global,
                format!("Scope prefix URL was not parseable: {scope_prefix}"),
            );
            // Step 2.3.2 Continue.
            continue;
        };

        // Step 2.4 Let normalizedScopePrefix be the serialization of scopePrefixURL.
        let normalized_scope_prefix = scope_prefix_url;

        // Step 2.5 Set normalized[normalizedScopePrefix] to the result of sorting and
        // normalizing a module specifier map given potentialSpecifierMap and baseURL.
        let normalized_specifier_map =
            sort_and_normalize_module_specifier_map(cx, global, potential_specifier_map, base_url);
        normalized.insert(normalized_scope_prefix, normalized_specifier_map);
    }

    // Step 3. Return the result of sorting in descending order normalized,
    // with an entry a being less than an entry b if a's key is code unit less than b's key.
    normalized.sort_by(|a_key, _, b_key, _| b_key.cmp(a_key));
    Ok(normalized)
}

/// <https://html.spec.whatwg.org/multipage/#normalizing-a-module-integrity-map>
fn normalize_module_integrity_map(
    cx: &mut JSContext,
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
) -> ModuleIntegrityMap {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized = ModuleIntegrityMap::new();

    // Step 2. For each key → value of originalMap:
    for (key, value) in original_map {
        // Step 2.1 Let resolvedURL be the result of
        // resolving a URL-like module specifier given key and baseURL.
        let Some(resolved_url) = resolve_url_like_module_specifier(key.as_str(), base_url) else {
            // Step 2.2 If resolvedURL is null, then:
            // Step 2.2.1 The user agent may report a warning
            // to the console indicating that the key failed to resolve.
            Console::internal_warn(
                cx,
                global,
                format!("Key failed to resolve to module specifier: {key}"),
            );
            // Step 2.2.2 Continue.
            continue;
        };

        // Step 2.3 If value is not a string, then:
        let JsonValue::String(value) = value else {
            // Step 2.3.1 The user agent may report a warning
            // to the console indicating that integrity metadata values need to be strings.
            Console::internal_warn(
                cx,
                global,
                "Integrity metadata values need to be strings.".to_string(),
            );
            // Step 2.3.2 Continue.
            continue;
        };

        // Step 2.4 Set normalized[resolvedURL] to value.
        normalized.insert(resolved_url, value.clone());
    }

    // Step 3. Return normalized.
    normalized
}

/// <https://html.spec.whatwg.org/multipage/#normalizing-a-specifier-key>
fn normalize_specifier_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    specifier_key: &str,
    base_url: &ServoUrl,
) -> Option<String> {
    // step 1. If specifierKey is the empty string, then:
    if specifier_key.is_empty() {
        // step 1.1 The user agent may report a warning to the console
        // indicating that specifier keys may not be the empty string.
        Console::internal_warn(
            cx,
            global,
            "Specifier keys may not be the empty string.".to_string(),
        );
        // step 1.2 Return null.
        return None;
    }
    // step 2. Let url be the result of resolving a URL-like module specifier, given specifierKey and baseURL.
    let url = resolve_url_like_module_specifier(specifier_key, base_url);

    // step 3. If url is not null, then return the serialization of url.
    if let Some(url) = url {
        return Some(url.into_string());
    }

    // step 4. Return specifierKey.
    Some(specifier_key.to_string())
}

/// <https://html.spec.whatwg.org/multipage/#resolving-a-url-like-module-specifier>
pub(crate) fn resolve_url_like_module_specifier(
    specifier: &str,
    base_url: &ServoUrl,
) -> Option<ServoUrl> {
    // Step 1. If specifier starts with "/", "./", or "../", then:
    if specifier.starts_with('/') || specifier.starts_with("./") || specifier.starts_with("../") {
        // Step 1.1. Let url be the result of URL parsing specifier with baseURL.
        return ServoUrl::parse_with_base(Some(base_url), specifier).ok();
    }
    // Step 2. Let url be the result of URL parsing specifier (with no base URL).
    ServoUrl::parse(specifier).ok()
}
