use std::rc::Rc;

use webdriver_traits::bidi::{
    StorageCommand, StorageResult,
    storage::{
        DeleteCookiesParameters, DeleteCookiesResult, GetCookiesParameters, GetCookiesResult,
        SetCookieParameters, SetCookieResult,
    },
};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_storage_command(
        self: Rc<Self>,
        command: StorageCommand,
    ) -> BidiResult<StorageResult> {
        match command {
            StorageCommand::GetCookies(cmd) => self
                .handle_storage_get_cookies(cmd.params)
                .await
                .map(StorageResult::GetCookiesResult),
            StorageCommand::SetCookie(cmd) => self
                .handle_storage_set_cookie(cmd.params)
                .await
                .map(StorageResult::SetCookieResult),
            StorageCommand::DeleteCookies(cmd) => self
                .handle_storage_delete_cookies(cmd.params)
                .await
                .map(StorageResult::DeleteCookiesResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-getGookies>
    async fn handle_storage_get_cookies(
        self: Rc<Self>,
        command_parameters: GetCookiesParameters,
    ) -> BidiResult<GetCookiesResult> {
        // 1.
        let filter = command_parameters.filter.clone().unwrap_or_default();
        // 2.
        let partition_spec = &command_parameters.partition;
        // 3.
        // let partition_key = self
        //     .expand_a_storage_partition(partition_spec.clone())
        //     .await?;
        // // TODO: 4-7. may should happen in resource thread.
        // // here we do not need to query script thread because associated storage partition is synced before.
        // // cookie store can be defined as a key-channel pair to resource thread
        // // 4. TODO: get cookie store
        // // 5. TODO: get matching cookies
        // // 6.
        // // 7.
        // let serialized_cookies = vec![];
        // // 8.
        // let body = storage::GetCookiesResult {
        //     cookies: serialized_cookies,
        //     partition_key,
        // };
        // // 9.
        // Ok(body)
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-setCookie>
    async fn handle_storage_set_cookie(
        self: Rc<Self>,
        command_parameters: SetCookieParameters,
    ) -> BidiResult<SetCookieResult> {
        // 1.
        let cookie_spec = &command_parameters.cookie;
        // 2.
        let partition_sepc = command_parameters.partition.clone();
        // 3.
        // let partition_key = self.expand_a_storage_partition(partition_sepc).await?;
        // // 4.
        // // 5-6. TODO: continue in resource thread
        // // 7.
        // let body = SetCookieResult { partition_key };
        // // 8.
        // Ok(body)
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-deleteCookies>
    async fn handle_storage_delete_cookies(
        self: Rc<Self>,
        command_parameters: DeleteCookiesParameters,
    ) -> BidiResult<DeleteCookiesResult> {
        // 1.
        let filter = command_parameters.filter.clone().unwrap_or_default();
        // 2.
        let partition_spec = &command_parameters.partition;
        // 3.
        // let partition_key = self
        //     .expand_a_storage_partition(partition_spec.clone())
        //     .await?;
        // // 4-6. TODO: continue in resource thread
        // // 7.
        // let body = storage::DeleteCookiesResult { partition_key };
        // // 8.
        // Ok(body)
        todo!()
    }
}
