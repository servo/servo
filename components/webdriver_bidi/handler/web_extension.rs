//! Web Extension is not implemented in servo yet. So this module is no-op.

use std::path::PathBuf;

use rustenium_bidi_definitions::{
    base::ErrorCode,
    web_extension::{
        commands::{InstallParams, UninstallParams, WebExtensionCommand},
        results::{InstallResult, UninstallResult},
        types::{Extension, ExtensionData},
    },
};
use uuid::Uuid;

use crate::{
    error::WebDriverBidiError,
    handler::{
        Handler,
        common::{DirectoryEntry, DirectoryLocator, FileLocator},
    },
    model::WebExtensionResult,
};

// NOTE: Servo currently does not support web extensions.
const IS_WEB_EXTENSION_SUPPORTED: bool = false;

impl Handler {
    pub(super) async fn handle_web_extension(
        &self,
        cmd: &WebExtensionCommand,
    ) -> Result<WebExtensionResult, WebDriverBidiError> {
        match cmd {
            WebExtensionCommand::Install(cmd) => self
                .handle_web_extension_install(&cmd.params)
                .await
                .map(WebExtensionResult::Install),
            WebExtensionCommand::Uninstall(cmd) => self
                .handle_web_extension_uninstall(&cmd.params)
                .await
                .map(WebExtensionResult::Uninstall),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-webExtension-install>
    async fn handle_web_extension_install(
        &self,
        command_parameters: &InstallParams,
    ) -> Result<InstallResult, WebDriverBidiError> {
        // 1. If installing web extensions isn’t supported return [error] with error code [unsupported operation].
        if !IS_WEB_EXTENSION_SUPPORTED {
            return Err(WebDriverBidiError::new(
                ErrorCode::UnsupportedOperation,
                "Servo has not supported web extension yet",
            ));
        }

        // 2. Let `extension data spec` be `command parameters["extensionData"]``.
        let extension_data_spec = &command_parameters.extension_data;

        // 3. Let `extension directory entry` be the result of [trying] to [expand a web extension data spec] with `extension
        // data spec`.
        let extension_directory_entry = self
            .expand_web_extension_data_sepc(extension_data_spec)
            .await?;

        // 4. If `extension directory entry` is null, return error with error code invalid web extension.
        let Some(extension_directory_entry) = extension_directory_entry else {
            return Err(WebDriverBidiError::new(
                ErrorCode::InvalidWebExtension,
                "Invalid web extension",
            ));
        };

        // 5. Perform implementation defined steps to install a web extension from `extension directory entry`. If this
        // fails, return [error] with [error code invalid web extension]. Otherwise let `extension id` be the unique
        // identifier of the newly installed web extension.
        let extension_id = match self.install_web_extension(&extension_directory_entry).await {
            Ok(extension_id) => extension_id,
            Err(message) => {
                return Err(WebDriverBidiError::new(
                    ErrorCode::InvalidWebExtension,
                    message,
                ));
            },
        };

        // 6. Let `result` be a [map] matching the `webExtension.InstallResult` production with the `extension` field
        // set to `extension id`.
        let result = InstallResult {
            extension: Extension::new(extension_id),
        };

        // 7. Return [success] with data `result`.
        Ok(result)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-webExtension-uninstall>
    async fn handle_web_extension_uninstall(
        &self,
        command_parameters: &UninstallParams,
    ) -> Result<UninstallResult, WebDriverBidiError> {
        // 1. Let `extension` be `command parameters["extension"]`.
        let extension = &command_parameters.extension;

        // 2. If the [remote end] has no web extension with id equal to `extension`, return [error] with [error code] [no such
        // web extension].
        let has_no_web_extension = true;
        if has_no_web_extension {
            return Err(WebDriverBidiError::new(
                ErrorCode::NoSuchWebExtension,
                "Servo has not supported web extensions yet, so there is no such web extension",
            ));
        }

        // 3. Perform any implementation-defined steps to remove the web extension from the [remote end]. If this
        // fails, return [error] with [error code] [unknown error].
        if let Err(message) = self.remove_web_extension(extension).await {
            return Err(WebDriverBidiError::unknown(message));
        }

        // 4. Return success with data null.
        Ok(UninstallResult {
            extensible: Default::default(),
        })
    }
}

impl Handler {
    /// To expand a web extension data spec given `extension data spec`:
    async fn expand_web_extension_data_sepc(
        &self,
        extension_data_spec: &ExtensionData,
    ) -> Result<Option<DirectoryEntry>, WebDriverBidiError> {
        // 1. Let `type` be `extension data spec["type"]`.
        // SKIP: use match in step 3 instead.

        // 2. If installing a web extension using `type` isn’t supported return [error] with [error code unsupported
        // operation].
        if !IS_WEB_EXTENSION_SUPPORTED {
            return Err(WebDriverBidiError::new(
                ErrorCode::UnsupportedOperation,
                "installing a web extension using type isn't supported",
            ));
        }

        // 3. In the following list of conditions and associated steps, run the first set of steps for which the associated
        // condition is true:
        let entry: Option<DirectoryEntry>;
        match extension_data_spec {
            // `type` is string "path"
            ExtensionData::ExtensionPath(extension_data_spec) => {
                // 3.1. Let `path` be `extension data spec["path"]`.
                let path = &extension_data_spec.path;

                // 3.2. Let `locator` be a [directory locator] with [path] `path` and [root] corresponding to the root of the file
                // system.
                let locator = DirectoryLocator {
                    path: PathBuf::from(path),
                    root: String::new(),
                };

                // 3.3. Let `entry` be [locate an entry] given `locator`.
                entry = self.locate_entry(&locator).await;
            },
            ExtensionData::ExtensionArchivePath(extension_data_spec) => {
                // 3.1. Let `archive path` be `extension data spec["path"]`.
                let archive_path = &extension_data_spec.path;

                // 3.2. Let `locator` be a [file locator] with [path] `archive path` and [root] corresponding to the root of the
                // file system.
                let locator = FileLocator {
                    path: PathBuf::from(archive_path),
                    root: String::new(),
                };

                // 3.3. Let `archive entry` be [locate an entry] given `locator`.
                let archive_entry = self.locate_entry(&locator).await;

                // 3.4. If `archive entry` is null, return null.
                let Some(archive_entry) = archive_entry else {
                    return Ok(None);
                };

                // 3.5. Let `bytes` be `archive entry`’s [binary data].
                let bytes = &archive_entry.binary_data;

                // 3.6. Let `entry` be the result of [trying] to [extract a zip archive] given `bytes`.
                entry = Some(self.extract_zip_archive(bytes).await?);
            },
            ExtensionData::ExtensionBase64Encoded(extension_data_spec) => {
                // 3.1. Let `bytes` be [forgiving-base64 decode] `extension data spec["value"]`.
                let bytes = self
                    .forgiving_base64_decode(&extension_data_spec.value)
                    .await;

                // 3.2. If bytes is failure, return null.
                let Ok(bytes) = bytes else {
                    return Ok(None);
                };

                // 3.3. Let entry be the result of trying to extract a zip archive given bytes.
                entry = Some(self.extract_zip_archive(&bytes).await?)
            },
        };

        // 4. Return `entry`.
        Ok(entry)
    }

    /// To extract a zip archive given `bytes`:
    async fn extract_zip_archive(
        &self,
        _bytes: &[u8],
    ) -> Result<DirectoryEntry, WebDriverBidiError> {
        // 1. Perform implementation defined steps to decode `bytes` using the zip compression algorithm.

        // 2. If the previous step failed (e.g. because `bytes` did not represent valid zip-compressed data) then return
        // [error] with error code [invalid web extension]. Otherwise let `entry` be a [directory entry] containing
        // the extracted filesystem entries.

        // 3. Return `entry`.
        unimplemented!()
    }

    async fn forgiving_base64_decode(&self, _base64: &str) -> Result<Vec<u8>, ()> {
        unimplemented!()
    }

    async fn install_web_extension(
        &self,
        _extension_directory_entry: &DirectoryEntry,
    ) -> Result<Uuid, &'static str> {
        Err("Servo has not supported web extensions yet")
    }

    async fn remove_web_extension(&self, _extension: &Extension) -> Result<(), &'static str> {
        Err("Servo has not supported web extensions yet")
    }
}
