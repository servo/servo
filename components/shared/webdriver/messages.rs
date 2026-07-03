use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_base::{generic_channel::GenericSender, id::WebViewId};

use crate::{
    bidi::{
        Command, ErrorCode, Message,
        script::{
            Channel, ChannelValue, ExceptionDetails, Handle, LocalValue, RealmInfo, RemoteValue,
            ResultOwnership, SerializationOptions,
        },
    },
    ids::{ConnectionId, PreloadScriptId, RealmId, ResumeId, SessionId},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EmbedderToWebDriverMessage {
    /// Create a new connection to webdriver.
    Connection(ConnectionId, Option<SessionId>),
    /// Send command to webdriverr
    Command(ConnectionId, Command),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToEmbedderMessage {
    /// The response to command, or event.
    Message(ConnectionId, Message),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToConstellationMessage {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConstellationToWebDriverMessage {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToScriptMessage {
    /// Request the realm to disown handles specified.
    Disown(ResumeId, RealmId, Vec<Handle>),
    /// Request the realm to evaluate JS string.
    Evaluate(ResumeId, RealmId, EvaluateBody),
    /// Request the realm to evaluate JS function.
    CallFunction(ResumeId, RealmId, CallFunctionBody),
    AddPreloadScripts(RealmId, Vec<(PreloadScriptId, PreloadScriptBody)>),
    RemovePreloadScripts(RealmId, Vec<PreloadScriptId>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    /// When a realm is created, it should notify webdriver.
    RealmCreated(
        RealmInfo,
        bool,
        Option<WebViewId>,
        GenericSender<WebDriverToScriptMessage>,
    ),
    /// When a realm is (to be) destroyed, it should notify webdriver.
    RealmDestroyed(RealmId),
    /// Response for [`WebDriverToScriptMessage::Disown`].
    Disowned(ResumeId),
    /// Response for [`WebDriverToScriptMessage::Evaluate`]
    /// and [`WebDriverToScriptMessage::CallFunction`].
    Evaluated(ResumeId, Result<EvaluationResultBody, ErrorCode>),
    Message(MessageBody),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverMessage {
    FromConstellation(ConstellationToWebDriverMessage),
    FromScript(ScriptToWebDriverMessage),
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PreloadScriptBody {
    pub function_declaration: String,
    pub arguments: Vec<ChannelValue>,
    pub sandbox: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CallFunctionBody {
    pub function_declaration: String,
    pub await_promise: bool,
    pub arguments: Vec<LocalValue>,
    pub result_ownership: ResultOwnership,
    pub serialization_options: SerializationOptions,
    pub this: Option<LocalValue>,
    pub user_activation: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvaluateBody {
    pub expression: String,
    pub await_promise: bool,
    pub result_ownership: ResultOwnership,
    pub serialization_options: SerializationOptions,
    pub user_activation: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EvaluationResultBody {
    Success(RemoteValue),
    Exception(ExceptionDetails),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageBody {
    pub channel: Channel,
    pub data: RemoteValue,
    pub realm: RealmId,
}
