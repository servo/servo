/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};

use wasmtime::component::Component;
use wasmtime::{Config, Engine};

/// Global, process-wide Wasm engine singleton
pub static SERVO_WASM_ENGINE: LazyLock<Arc<Engine>> = LazyLock::new(|| {
    let mut config = Config::new();
    config.wasm_component_model(true);

    // Optional: Enables fast memory bounds checking
    config.cranelift_opt_level(wasmtime::OptLevel::Speed);

    let engine = Engine::new(&config).expect("[Servo Wasm]: Failed to initialize Wasmtime Engine");
    Arc::new(engine)
});

/// Thread-safe component cache keyed by URI, path, or content hash
pub struct ComponentRegistry {
    cache: RwLock<HashMap<PathBuf, Arc<Component>>>,
}

pub static COMPONENT_REGISTRY: LazyLock<ComponentRegistry> = LazyLock::new(|| ComponentRegistry {
    cache: RwLock::new(HashMap::new()),
});

impl ComponentRegistry {
    /// Get or compile a component from a file path
    pub fn get_or_load_file(&self, path: impl AsRef<Path>) -> wasmtime::Result<Arc<Component>> {
        let path = path.as_ref().to_path_buf();

        // Fast read lock check
        if let Some(comp) = self.cache.read().unwrap().get(&path) {
            return Ok(comp.clone());
        }

        // Compile and write lock
        let mut writer = self.cache.write().unwrap();
        if let Some(comp) = writer.get(&path) {
            return Ok(comp.clone());
        }

        let engine = &*SERVO_WASM_ENGINE;
        let component = Arc::new(Component::from_file(engine, &path)?);
        writer.insert(path, component.clone());
        Ok(component)
    }

    /// Get or compile a component from downloaded bytes (URL fetch)
    pub fn get_or_load_binary(
        &self,
        key: impl AsRef<Path>,
        bytes: &[u8],
    ) -> wasmtime::Result<Arc<Component>> {
        let key = key.as_ref().to_path_buf();
        if let Some(comp) = self.cache.read().unwrap().get(&key) {
            return Ok(comp.clone());
        }

        let mut writer = self.cache.write().unwrap();
        if let Some(comp) = writer.get(&key) {
            return Ok(comp.clone());
        }

        let engine = &*SERVO_WASM_ENGINE;
        let component = Arc::new(Component::from_binary(engine, bytes)?);
        writer.insert(key, component.clone());
        Ok(component)
    }
}
