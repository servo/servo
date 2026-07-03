// hand-written version

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    pub id: u64,
    pub command_data: CommandData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CommandData {
    Session(SessionCommand),
    Script(ScriptCommand),
    Unknown(Value),
}

impl CommandData {
    pub fn is_static(&self) -> bool {
        match self {
            CommandData::Session(cmd) => match cmd {
                SessionCommand::New(_) | SessionCommand::Status(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EmptyParams {
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "success")]
    CommandResponse(CommandResponse),
    #[serde(rename = "error")]
    ErrorResponse(ErrorResponse),
    #[serde(rename = "event")]
    Event(Event),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommandResponse {
    pub id: u64,
    pub result: ResultData,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub id: Option<u64>,
    pub error: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResultData {
    Session(SessionResult),
    Script(ScriptResult),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EmptyResult {
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    #[serde(flatten)]
    pub event_data: EventData,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EventData {
    #[serde(untagged)]
    Script(ScriptEvent),
}

pub type Extensible = HashMap<String, Value>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ErrorCode {
    #[serde(rename = "invalid argument")]
    InvalidArgument,
    #[serde(rename = "invalid selector")]
    InvalidSelector,
    #[serde(rename = "invalid session id")]
    InvalidSessionId,
    #[serde(rename = "invalid web extension")]
    InvalidWebExtension,
    #[serde(rename = "move target out of bounds")]
    MoveTargetOutOfBounds,
    #[serde(rename = "no such alert")]
    NoSuchAlert,
    #[serde(rename = "no such network collector")]
    NoSuchNetworkCollector,
    #[serde(rename = "no such element")]
    NoSuchElement,
    #[serde(rename = "no such frame")]
    NoSuchFrame,
    #[serde(rename = "no such handle")]
    NoSuchHandle,
    #[serde(rename = "no such history entry")]
    NoSuchHistoryEntry,
    #[serde(rename = "no such intercept")]
    NoSuchIntercept,
    #[serde(rename = "no such network data")]
    NoSuchNetworkData,
    #[serde(rename = "no such node")]
    NoSuchNode,
    #[serde(rename = "no such request")]
    NoSuchRequest,
    #[serde(rename = "no such screencast")]
    NoSuchScreencast,
    #[serde(rename = "no such script")]
    NoSuchScript,
    #[serde(rename = "no such storage partition")]
    NoSuchStoragePartition,
    #[serde(rename = "no such user context")]
    NoSuchUserContext,
    #[serde(rename = "no such web extension")]
    NoSuchWebExtension,
    #[serde(rename = "session not created")]
    SessionNotCreated,
    #[serde(rename = "unable to capture screen")]
    UnableToCaptureScreen,
    #[serde(rename = "unable to close browser")]
    UnableToCloseBrowser,
    #[serde(rename = "unable to set cookie")]
    UnableToSetCookie,
    #[serde(rename = "unable to set file input")]
    UnableToSetFileInput,
    #[serde(rename = "unavailable network data")]
    UnavailableNetworkData,
    #[serde(rename = "underspecified storage partition")]
    UnderspecifiedStoragePartition,
    #[serde(rename = "unknown command")]
    UnknownCommand,
    #[serde(rename = "unknown error")]
    UnknownError,
    #[serde(rename = "unsupported operation")]
    UnsupportedOperation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "method")]
pub enum SessionCommand {
    #[serde(rename = "session.end")]
    End(session::End),
    #[serde(rename = "session.new")]
    New(session::New),
    #[serde(rename = "session.status")]
    Status(session::Status),
    #[serde(rename = "session.subscribe")]
    Subscribe(session::Subscribe),
    #[serde(rename = "session.unsubscribe")]
    Unsubscribe(session::Unsubscribe),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "method")]
pub enum SessionResult {
    #[serde(rename = "session.end")]
    End(session::EndResult),
    #[serde(rename = "session.new")]
    New(session::NewResult),
    #[serde(rename = "session.status")]
    Status(session::StatusResult),
    #[serde(rename = "session.subscribe")]
    Subscribe(session::SubscribeResult),
    #[serde(rename = "session.unsubscribe")]
    Unsubscribe(session::UnsubscribeResult),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "method")]
pub enum ScriptCommand {
    #[serde(rename = "script.addPreloadScript")]
    AddPreloadScript(script::AddPreloadScript),
    #[serde(rename = "script.callFunction")]
    CallFunction(script::CallFunction),
    #[serde(rename = "script.disown")]
    Disown(script::Disown),
    #[serde(rename = "script.evaluate")]
    Evaluate(script::Evaluate),
    #[serde(rename = "script.getRealms")]
    GetRealms(script::GetRealms),
    #[serde(rename = "script.removePreloadScript")]
    RemovePreloadScript(script::RemovePreloadScript),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "method")]
pub enum ScriptResult {
    #[serde(rename = "script.addPreloadScript")]
    AddPreloadScript(script::AddPreloadScriptResult),
    #[serde(rename = "script.callFunction")]
    CallFunction(script::CallFunctionResult),
    #[serde(rename = "script.disown")]
    Disown(script::DisownResult),
    #[serde(rename = "script.evaluate")]
    Evaluate(script::EvaluateResult),
    #[serde(rename = "script.getRealms")]
    GetRealms(script::GetRealmsResult),
    #[serde(rename = "script.removePreloadScript")]
    RemovePreloadScript(script::RemovePreloadScriptResult),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "method")]
pub enum ScriptEvent {
    #[serde(rename = "script.message")]
    Message(script::Message),
    #[serde(rename = "script.realmCreated")]
    RealmCreated(script::RealmCreated),
    #[serde(rename = "script.realmDestroyed")]
    RealmDestroyed(script::RealmDestroyed),
}

pub mod session {
    use serde::{Deserialize, Serialize};

    use crate::{
        bidi::{EmptyParams, EmptyResult, Extensible, browser, browsing_context},
        ids::SessionId,
    };

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CapabilitiesRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub always_match: Option<CapabilityRequest>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub first_match: Option<Vec<CapabilityRequest>>,
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CapabilityRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub accept_insecure_certs: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub browser_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub browser_version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub platform_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub proxy: Option<ProxyConfiguration>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub unhandled_prompt_behavior: Option<String>,
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "proxyType", rename_all = "camelCase")]
    pub enum ProxyConfiguration {
        Autodetect(AutodetectProxyConfiguration),
        Direct(DirectProxyConfiguration),
        Manual(ManualProxyConfiguration),
        Pac(PacProxyConfiguration),
        System(SystemProxyConfiguration),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AutodetectProxyConfiguration {
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DirectProxyConfiguration {
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManualProxyConfiguration {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub http_proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ssl_proxy: Option<String>,
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pub socks: Option<SocksProxyConfiguration>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub no_proxy: Option<Vec<String>>,
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SocksProxyConfiguration {
        pub socks_proxy: String,
        pub socks_version: u8,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PacProxyConfiguration {
        pub proxy_autoconfig_url: String,
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SystemProxyConfiguration {
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UserPromptHandler {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub alert: Option<UserPromptHandlerType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub before_unload: Option<UserPromptHandlerType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub confirm: Option<UserPromptHandlerType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<UserPromptHandlerType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub file: Option<UserPromptHandlerType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prompt: Option<UserPromptHandlerType>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum UserPromptHandlerType {
        Accept,
        Dismiss,
        Ignore,
    }

    pub type Subscription = crate::ids::SubscriptionId;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubscribeParameters {
        pub events: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub contexts: Option<Vec<browsing_context::BrowsingContext>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_contexts: Option<Vec<browser::UserContext>>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct UnsubscribeByIdRequest {
        pub subscriptions: Vec<Subscription>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct UnsubscribeByAttributesRequest {
        pub events: Vec<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Status {
        pub params: EmptyParams,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct StatusResult {
        pub ready: bool,
        pub message: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct New {
        pub params: NewParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NewParameters {
        pub capabilities: CapabilitiesRequest,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NewResult {
        pub session_id: SessionId,
        pub capabilities: NewResultCapabilities,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NewResultCapabilities {
        pub accept_insecure_certs: bool,
        pub browser_name: String,
        pub browser_version: String,
        pub platform_name: String,
        pub set_window_rect: bool,
        pub user_agent: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub proxy: Option<ProxyConfiguration>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub unhandled_prompt_behavior: Option<UserPromptHandler>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub websocket_url: Option<String>,
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct End {
        pub params: EmptyParams,
    }

    pub type EndResult = EmptyResult;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Subscribe {
        pub params: SubscribeParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SubscribeResult {
        pub subscription: Subscription,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Unsubscribe {
        pub params: UnsubscribeParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum UnsubscribeParameters {
        ByAttributes(UnsubscribeByAttributesRequest),
        ById(UnsubscribeByIdRequest),
    }

    pub type UnsubscribeResult = EmptyResult;
}

pub mod browser {
    pub type UserContext = String;
}

pub mod browsing_context {
    pub type BrowsingContext = servo_base::id::BrowsingContextId;
}

pub mod script {
    use std::collections::HashMap;

    use malloc_size_of_derive::MallocSizeOf;
    use serde::{Deserialize, Serialize};

    use crate::{
        bidi::{EmptyResult, Extensible, browser, browsing_context},
        ids::RealmId,
    };

    pub type Channel = String;

    #[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
    pub struct ChannelValue {
        pub value: ChannelProperties,
    }

    #[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChannelProperties {
        pub channel: Channel,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub serialization_options: Option<SerializationOptions>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub ownership: Option<ResultOwnership>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum EvaluateResult {
        Success(EvaluateResultSuccess),
        Exception(EvaluateResultException),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct EvaluateResultSuccess {
        pub result: RemoteValue,
        pub realm: Realm,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct EvaluateResultException {
        pub exception_details: ExceptionDetails,
        pub realm: Realm,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExceptionDetails {
        pub column_number: u32,
        pub exception: RemoteValue,
        pub line_number: u32,
        pub stack_trace: StackTrace,
        pub text: String,
    }

    pub type Handle = crate::ids::HandleId;

    pub type InternalId = crate::ids::InternalId;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum LocalValue {
        Channel(ChannelValue),
        Array(ArrayLocalValue),
        Date(DateLocalValue),
        Map(MapLocalValue),
        Object(ObjectLocalValue),
        Regexp(RegExpLocalValue),
        Set(SetLocalValue),
        #[serde(untagged)]
        RemoteReference(RemoteReference),
        #[serde(untagged)]
        PrimitiveProtocol(PrimitiveProtocolValue),
    }

    pub type ListLocalValue = Vec<LocalValue>;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ArrayLocalValue {
        pub value: ListLocalValue,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DateLocalValue {
        pub value: String,
    }

    pub type MappingLocalValue = Vec<(LocalValueOrText, LocalValue)>;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum LocalValueOrText {
        LocalValue(LocalValue),
        Text(String),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct MapLocalValue {
        pub value: MappingLocalValue,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ObjectLocalValue {
        pub value: MappingLocalValue,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RegExpValue {
        pub pattern: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub flags: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RegExpLocalValue {
        pub value: RegExpValue,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SetLocalValue {
        pub value: ListLocalValue,
    }

    pub type PreloadScript = crate::ids::PreloadScriptId;

    // Here we use uuid wrapper rather than raw string.
    pub type Realm = RealmId;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", tag = "type")]
    pub enum PrimitiveProtocolValue {
        Undefined,
        Null,
        String(StringValue),
        Number(NumberValue),
        Boolean(BooleanValue),
        Bigint(BigIntValue),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct StringValue {
        pub value: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub enum SpecialNumber {
        #[serde(rename = "NaN")]
        Nan,
        #[serde(rename = "-0")]
        NegZero,
        #[serde(rename = "Infinity")]
        Infinity,
        #[serde(rename = "-Infinity")]
        NegInfinity,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct NumberValue {
        pub value: NumberValueKind,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum NumberValueKind {
        Number(f64),
        SpecialNumber(SpecialNumber),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BooleanValue {
        pub value: bool,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BigIntValue {
        pub value: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "kebab-case", tag = "type")]
    pub enum RealmInfo {
        Window(WindowRealmInfo),
        DedicatedWorker(DedicatedWorkerRealmInfo),
        SharedWorker(SharedWorkerRealmInfo),
        ServiceWorker(ServiceWorkerRealmInfo),
        Worker(WorkerRealmInfo),
        PaintWorklet(PaintWorkletRealmInfo),
        AudioWorklet(AudioWorkletRealmInfo),
        Worklet(WorkletRealmInfo),
    }

    impl RealmInfo {
        pub fn realm(&self) -> &RealmId {
            match self {
                RealmInfo::Window(info) => &info.base.realm,
                RealmInfo::DedicatedWorker(info) => &info.base.realm,
                RealmInfo::SharedWorker(info) => &info.base.realm,
                RealmInfo::ServiceWorker(info) => &info.base.realm,
                RealmInfo::Worker(info) => &info.base.realm,
                RealmInfo::PaintWorklet(info) => &info.base.realm,
                RealmInfo::AudioWorklet(info) => &info.base.realm,
                RealmInfo::Worklet(info) => &info.base.realm,
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BaseRealmInfo {
        pub realm: Realm,
        pub origin: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct WindowRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
        pub context: browsing_context::BrowsingContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_context: Option<browser::UserContext>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sandbox: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DedicatedWorkerRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
        pub owners: Vec<Realm>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SharedWorkerRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ServiceWorkerRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct WorkerRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PaintWorkletRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AudioWorkletRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct WorkletRealmInfo {
        #[serde(flatten)]
        pub base: BaseRealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "kebab-case", tag = "type")]
    pub enum RealmType {
        Window,
        DedicatedWorker,
        SharedWorker,
        ServiceWorker,
        Worker,
        PaintWorklet,
        AudioWorklet,
        Worklet,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum RemoteReference {
        Shared(SharedReference),
        RemoteObject(RemoteObjectReference),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SharedReference {
        pub shared_id: SharedId,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RemoteObjectReference {
        pub handle: Handle,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub shared_id: Option<SharedId>,
        #[serde(flatten)]
        pub extensible: Extensible,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", tag = "type")]
    pub enum RemoteValue {
        Symbol(SymbolRemoteValue),
        Array(ArrayRemoteValue),
        Object(ObjectRemoteValue),
        Function(FunctionRemoteValue),
        #[serde(rename = "rexexp")]
        RegExp(RegExpRemoteValue),
        Date(DateRemoteValue),
        Map(MapRemoteValue),
        Set(SetRemoteValue),
        #[serde(rename = "weakmap")]
        WeakMap(WeakMapRemoteValue),
        #[serde(rename = "weakset")]
        WeakSet(WeakSetRemoteValue),
        Generator(GeneratorRemoteValue),
        Error(ErrorRemoteValue),
        Proxy(ProxyRemoteValue),
        Promise(PromiseRemoteValue),
        #[serde(rename = "typedarray")]
        TypedArray(TypedArrayRemoteValue),
        #[serde(rename = "arraybuffer")]
        ArrayBuffer(ArrayBufferRemoteValue),
        #[serde(rename = "nodelist")]
        NodeList(NodeListRemoteValue),
        #[serde(rename = "htmlcollection")]
        HtmlCollection(HtmlCollectionRemoteValue),
        Node(NodeRemoteValue),
        Window(WindowProxyRemoteValue),
        #[serde(untagged)]
        PrimitiveProtocol(PrimitiveProtocolValue),
    }

    pub type ListRemoteValue = Vec<RemoteValue>;

    pub type MappingRemoteValue = Vec<(RemoteValueOrText, RemoteValue)>;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum RemoteValueOrText {
        Value(RemoteValue),
        Text(String),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SymbolRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ArrayRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<ListRemoteValue>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ObjectRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<MappingRemoteValue>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FunctionRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RegExpRemoteValue {
        #[serde(flatten)]
        pub local: RegExpLocalValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DateRemoteValue {
        #[serde(flatten)]
        pub local: DateLocalValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MapRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<MappingRemoteValue>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<ListRemoteValue>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WeakMapRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WeakSetRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GeneratorRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ErrorRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ProxyRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PromiseRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TypedArrayRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ArrayBufferRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NodeListRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<ListRemoteValue>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HtmlCollectionRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<ListRemoteValue>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NodeRemoteValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub shared_id: Option<SharedId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<NodeProperties>,
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NodeProperties {
        pub node_type: u16,
        pub child_node_count: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attributes: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub children: Option<Vec<NodeRemoteValue>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub local_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mode: Option<ShadowRootMode>,
        #[serde(rename = "namespaceURI", skip_serializing_if = "Option::is_none")]
        pub namespace_uri: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub shadow_root: Option<Box<NodeRemoteValue>>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum ShadowRootMode {
        Open,
        Closed,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", tag = "type")]
    pub struct WindowProxyRemoteValue {
        pub value: WindowProxyProperties,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub handle: Option<Handle>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub internal_id: Option<InternalId>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WindowProxyProperties {
        pub context: browsing_context::BrowsingContext,
    }

    #[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum ResultOwnership {
        Root,
        None,
    }

    #[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SerializationOptions {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_dom_depth: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_object_depth: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_shadow_tree: Option<IncludeShadowTree>,
    }

    #[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub enum IncludeShadowTree {
        #[default]
        None,
        Open,
        All,
    }

    pub type SharedId = String;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StackFrame {
        pub column_number: u64,
        pub function_name: String,
        pub line_number: u64,
        pub url: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StackTrace {
        pub call_frames: Vec<StackFrame>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Source {
        pub realm: Realm,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub context: Option<browsing_context::BrowsingContext>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_context: Option<browser::UserContext>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RealmTarget {
        pub realm: Realm,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ContextTarget {
        pub context: browsing_context::BrowsingContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sandbox: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum Target {
        Context(ContextTarget),
        Realm(RealmTarget),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AddPreloadScript {
        pub params: AddPreloadScriptParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AddPreloadScriptParameters {
        pub function_declaration: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub arguments: Option<Vec<ChannelValue>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub contexts: Option<Vec<browsing_context::BrowsingContext>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_contexts: Option<Vec<browser::UserContext>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sandbox: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AddPreloadScriptResult {
        pub script: PreloadScript,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Disown {
        pub params: DisownParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DisownParameters {
        pub handles: Vec<Handle>,
        pub target: Target,
    }

    pub type DisownResult = EmptyResult;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CallFunction {
        pub params: CallFunctionParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionParameters {
        pub function_declaration: String,
        pub await_promise: bool,
        pub target: Target,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub arguments: Option<Vec<LocalValue>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result_ownership: Option<ResultOwnership>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub serialization_options: Option<SerializationOptions>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub this: Option<LocalValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_activation: Option<bool>,
    }

    pub type CallFunctionResult = EvaluateResult;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Evaluate {
        pub params: EvaluateParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EvaluateParameters {
        pub expression: String,
        pub target: Target,
        pub await_promise: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result_ownership: Option<ResultOwnership>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub serialization_options: Option<SerializationOptions>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_activation: Option<bool>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct GetRealms {
        pub params: GetRealmsParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct GetRealmsParameters {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub context: Option<browsing_context::BrowsingContext>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub r#type: Option<RealmType>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct GetRealmsResult {
        pub realms: Vec<RealmInfo>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RemovePreloadScript {
        pub params: RemovePreloadScriptParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RemovePreloadScriptParameters {
        pub script: PreloadScript,
    }

    pub type RemovePreloadScriptResult = EmptyResult;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Message {
        pub params: MessageParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct MessageParameters {
        pub channel: Channel,
        pub data: RemoteValue,
        pub source: Source,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RealmCreated {
        pub params: RealmInfo,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RealmDestroyed {
        pub params: RealmDestroyedParameters,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct RealmDestroyedParameters {
        pub realm: Realm,
    }
}
