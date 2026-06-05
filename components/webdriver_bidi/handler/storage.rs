use rustenium_bidi_definitions::storage::commands::StorageCommand;

use crate::{error::WebDriverBidiError, handler::Handler, model::StorageResult};

impl Handler {
    pub(super) async fn handle_storage(
        &self,
        cmd: &StorageCommand,
    ) -> Result<StorageResult, WebDriverBidiError> {
        match cmd {
            StorageCommand::GetCookies(get_cookies) => self.handle_storage_get_cookies().await,
            StorageCommand::SetCookie(set_cookie) => self.handle_storage_set_cookie().await,
            StorageCommand::DeleteCookies(delete_cookies) => {
                self.handle_storage_delete_cookies().await
            },
        }
    }

    async fn handle_storage_get_cookies(&self) -> Result<StorageResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_storage_set_cookie(&self) -> Result<StorageResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_storage_delete_cookies(&self) -> Result<StorageResult, WebDriverBidiError> {
        todo!()
    }
}
