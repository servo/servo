/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Lightweight WebExtensions (Manifest v2 / v3) loader and runtime compatibility layer.

use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use embedder_traits::user_contents::{UserScript, UserStyleSheet};
use serde::Deserialize;
use url::Url;

/// Parsed `manifest.json` for WebExtensions.
#[derive(Clone, Debug, Deserialize)]
pub struct ExtensionManifest {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub content_scripts: Vec<ContentScriptDeclaration>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Content script block declaration in manifest.json.
#[derive(Clone, Debug, Deserialize)]
pub struct ContentScriptDeclaration {
    #[serde(default)]
    pub matches: Vec<String>,
    #[serde(default)]
    pub js: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
    pub run_at: Option<String>,
}

/// A loaded WebExtension bundle with scripts and stylesheets ready for injection.
pub struct LoadedExtension {
    pub manifest: ExtensionManifest,
    pub scripts: Vec<Rc<UserScript>>,
    pub stylesheets: Vec<Rc<UserStyleSheet>>,
}

/// Load an unpacked extension directory containing a `manifest.json`.
pub fn load_extension(extension_dir: &Path) -> Result<LoadedExtension, Box<dyn std::error::Error>> {
    let manifest_path = extension_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

    let mut scripts = Vec::new();
    let mut stylesheets = Vec::new();

    // Generate Chrome / WebExtensions runtime API bridge script
    let runtime_api_bridge = format!(
        r#"
        (function() {{
            if (!window.chrome) window.chrome = {{}};
            if (!window.chrome.runtime) {{
                window.chrome.runtime = {{
                    id: "{}",
                    name: "{}",
                    version: "{}",
                    getManifest: function() {{ return {}; }},
                    sendMessage: function(message, callback) {{
                        if (window.servoMessageRouter) {{
                            window.servoMessageRouter.postMessage("extension:runtime:sendMessage", message);
                        }}
                        if (typeof callback === 'function') {{
                            callback({{ received: true }});
                        }}
                    }},
                    onMessage: {{
                        _listeners: [],
                        addListener: function(fn) {{ this._listeners.push(fn); }},
                        removeListener: function(fn) {{
                            this._listeners = this._listeners.filter(function(l) {{ return l !== fn; }});
                        }}
                    }}
                }};
            }}
            if (!window.browser) window.browser = window.chrome;
        }})();
        "#,
        manifest.name.replace(' ', "_").to_lowercase(),
        manifest.name,
        manifest.version,
        manifest_content
    );
    scripts.push(Rc::new(UserScript::new(
        runtime_api_bridge,
        Some(manifest_path.clone()),
    )));

    // Load content scripts and stylesheets
    for entry in &manifest.content_scripts {
        for script_file in &entry.js {
            let path = extension_dir.join(script_file);
            if let Ok(content) = fs::read_to_string(&path) {
                scripts.push(Rc::new(UserScript::new(content, Some(path))));
            } else {
                log::warn!("Could not read extension script file: {:?}", path);
            }
        }
        for css_file in &entry.css {
            let path = extension_dir.join(css_file);
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(url) = Url::from_file_path(&path) {
                    stylesheets.push(Rc::new(UserStyleSheet::new(content, url)));
                }
            } else {
                log::warn!("Could not read extension stylesheet file: {:?}", path);
            }
        }
    }

    Ok(LoadedExtension {
        manifest,
        scripts,
        stylesheets,
    })
}
