use servo_webdriver::bidi::{
    StorageCommand, StorageResult,
    network::{BytesValue, Cookie as SerializedCookie},
    storage::{
        CookieFilter, DeleteCookiesParameters, DeleteCookiesResult, GetCookiesParameters,
        GetCookiesResult, PartitionDescriptor, PartitionKey, SetCookieParameters, SetCookieResult,
    },
};

use crate::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    pub(super) async fn handle_storage(
        &self,
        cmd: StorageCommand,
    ) -> Result<StorageResult, WebDriverBidiError> {
        match cmd {
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

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-getCookies>
    async fn handle_storage_get_cookies(
        &self,
        command_parameters: GetCookiesParameters,
    ) -> Result<GetCookiesResult, WebDriverBidiError> {
        // 1. Let `filter` be the value of the `filter` field of `command parameters` if it is present or an empty [map] if it
        // isn’t.
        let filter = command_parameters.filter.unwrap_or_else(|| CookieFilter {
            name: None,
            value: None,
            domain: None,
            path: None,
            size: None,
            http_only: None,
            secure: None,
            same_site: None,
            expiry: None,
            extensible: Default::default(),
        });

        // 2. Let `partition spec` be the value of the `partition` field of `command parameters` if it is present or null if it
        // isn’t.
        let partition_spec = command_parameters.partition;

        // 3. Let `partition key` be the result of [trying] to [expand a storage partition spec] with `partition spec`.
        let partition_key = self.expand_storage_partition_spec(&partition_spec).await?;

        // 4. Let `store` be the result of [trying] to [get the cookie store] with `partition key`.
        let store = self.get_cookie_store(&partition_key).await?;

        // 5. Let `cookies` be the result of [get matching cookies] with `store` and `filter`.
        let cookies = self.get_matching_cookies(&store, &filter).await?;

        // 6. Let `serialized cookies` be a new list.
        let mut serialized_cookies = Vec::<SerializedCookie>::new();

        // 7. For each `cookie` in `cookies`:
        for cookie in cookies {
            // 7.1. Let `serialized cookie` be the result of [serialize cookie] given `cookie`.
            let serialized_cookie = self.serialize_cookie(&cookie);

            // 7.2. Append `serialized cookie` to `serialized cookies`.
            serialized_cookies.push(serialized_cookie);
        }

        // 8. Let `body` be a [map] matching the `storage.GetCookiesResult` production, with the `cookies` field set to
        // `serialized cookies` and the `partitionKey` field set to `partition key`.
        let body = GetCookiesResult {
            cookies: serialized_cookies,
            partition_key,
        };

        // 9. Return [success] with data `body`.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-setCookie>
    async fn handle_storage_set_cookie(
        &self,
        command_parameters: SetCookieParameters,
    ) -> Result<SetCookieResult, WebDriverBidiError> {
        // 1. Let `cookie spec` be the value of the `cookie` field of `command parameters`.
        let cookie_spec = command_parameters.cookie;

        // 2. Let `partition spec` be the value of the `partition` field of `command parameters` if it is present or null if it
        // isn’t.
        let partition_spec = command_parameters.partition;

        // 3. Let `partition key` be the result of [trying] to [expand a storage partition spec] with `partition spec`.
        let partition_key = self.expand_storage_partition_spec(&partition_spec).await?;

        // 4. Let `store` be the result of [trying] to [get the cookie store] with `partition key`.
        let store = self.get_cookie_store(&partition_key).await?;

        // 5. Let `deserialized value` be [deserialize protocol bytes] with `cookie spec["value"]`.
        let deserialized_value = self.deserialize_protocol_bytes(&cookie_spec.value);

        // 6. [Create a cookie] in `store` using [cookie name] `cookie spec["name"]`, [cookie value] `deserialized value`, [cookie
        // domain] `cookie spec["domain"]`, and an attribute-value list of the following cookie concepts listed in the
        // [table for cookie conversion]:
        //
        // Cookie path
        // cookie spec["path"] if it exists, otherwise "/".
        //
        // Cookie secure only
        // cookie spec["secure"] if it exists, otherwise false.
        //
        // Cookie HTTP only
        // cookie spec["httpOnly"] if it exists, otherwise false.
        //
        // Cookie expiry time
        // cookie spec["expiry"] if it exists, otherwise leave unset to indicate that this is a session cookie.
        //
        // Cookie same site
        // cookie spec["sameSite"] if it exists, otherwise leave unset to indicate that no same site policy is defined.
        //
        // If this step is aborted without inserting a cookie into the cookie store, return error with error code unable to set cookie.
        self.create_a_cookie(
            &store,
            &cookie_spec.name,
            &deserialized_value,
            &cookie_spec.domain,
        );
        // TODO: cookie conversion

        // 7. Let `body` be a [map] matching the `storage.SetCookieResult` production, with the `partitionKey` field set
        // to `partition key`.
        let body = SetCookieResult { partition_key };

        // 8. Return [success] with data `body`.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-deleteCookies>
    async fn handle_storage_delete_cookies(
        &self,
        command_parameters: DeleteCookiesParameters,
    ) -> Result<DeleteCookiesResult, WebDriverBidiError> {
        // 1. Let `filter` be the value of the `filter` field of `command parameters` if it is present or an empty [map] if it
        // isn’t.
        let filter = command_parameters.filter.unwrap_or_else(|| CookieFilter {
            name: None,
            value: None,
            domain: None,
            path: None,
            size: None,
            http_only: None,
            secure: None,
            same_site: None,
            expiry: None,
            extensible: Default::default(),
        });

        // 2. Let `partition spec` be the value of the `partition` field of `command parameters` if it is present or null if it
        // isn’t.
        let partition_spec = command_parameters.partition;

        // 3. Let `partition key` be the result of [trying] to [expand a storage partition spec] with `partition spec`.
        let partition_key = self.expand_storage_partition_spec(&partition_spec).await?;

        // 4. Let `store` be the result of [trying] to [get the cookie store] with `partition key`.
        let store = self.get_cookie_store(&partition_key).await?;

        // 5. Let `cookies` be the result of [get matching cookies] with `store` and `filter`.
        let cookies = self.get_matching_cookies(&store, &filter).await?;

        // 6. For each cookie in cookies:
        for cookie in cookies {
            // 6.1. Remove cookie from store.
            self.remote_cookie_from_store(&cookie);
        }

        // 7. Let `body` be a [map] matching the `storage.DeleteCookiesResult` production, with the `partitionKey`
        // field set to `partition key`.
        let body = DeleteCookiesResult { partition_key };

        // 8. Return [success] with data `body`.
        Ok(body)
    }
}

// TODO: to be implemented
pub struct Cookie;
pub struct Store;

// TODO: distinguish serialized cookie and cookie
impl Handler {
    /// <https://www.w3.org/TR/webdriver-bidi/#expand-a-storage-partition-spec>
    async fn expand_storage_partition_spec(
        &self,
        partition_spec: &Option<PartitionDescriptor>,
    ) -> Result<PartitionKey, WebDriverBidiError> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-the-cookie-store>
    async fn get_cookie_store(
        &self,
        partition_key: &PartitionKey,
    ) -> Result<Store, WebDriverBidiError> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-matching-cookies>
    async fn get_matching_cookies(
        &self,
        store: &Store,
        filter: &CookieFilter,
    ) -> Result<Vec<Cookie>, WebDriverBidiError> {
        todo!()
    }

    fn serialize_cookie(&self, cookie: &Cookie) -> SerializedCookie {
        todo!()
    }

    fn remote_cookie_from_store(&self, cookie: &Cookie) {
        todo!()
    }

    fn deserialize_protocol_bytes(&self, protocol_bytes: &BytesValue) -> String {
        todo!()
    }

    fn create_a_cookie(
        &self,
        store: &Store,
        cookie_name: &str,
        cookie_value: &str,
        cookie_domain: &str,
    ) {
        todo!()
    }
}
