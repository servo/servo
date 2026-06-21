mod connection;
mod constellation;
mod error;
mod modules;
mod remote_end;
mod script;
mod session;
mod util;

use std::{
    net::{SocketAddr, SocketAddrV4},
    thread::{self},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use embedder_traits::{EmbedderMsg, GenericEmbedderProxy};
use net_traits::ResourceThreads;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use webdriver_traits::{WebDriverMsg, WebDriverToConstellationMsg};

// TODO: this should later be renamed to `WebDriverServer`
// after classic is merged.
pub struct WebDriverBidiThread {
    port: u16,
    embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    constellation_sender: Sender<WebDriverToConstellationMsg>,
    resource_threads: ResourceThreads,
    // Remote end states are shared across all sessions.
    // Though this is a single threaded
}

impl WebDriverBidiThread {
    pub fn start(
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        resource_threads: ResourceThreads,
    ) -> (
        UnboundedSender<WebDriverMsg>,
        Receiver<WebDriverToConstellationMsg>,
    ) {
        let (c2w_sender, c2w_receiver) = mpsc::unbounded_channel();
        let (w2c_sender, w2c_receiver) = unbounded();

        thread::Builder::new()
            .name("WebDriverBiDi".to_string())
            .spawn(move || {
                WebDriverBidiThread::new(0, embedder_proxy, resource_threads, w2c_sender)
                    .run(c2w_receiver);
            })
            .expect("Thread spawning failed");

        (c2w_sender, w2c_receiver)
    }

    fn new(
        port: u16,
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        resource_threads: ResourceThreads,
        constellation_sender: Sender<WebDriverToConstellationMsg>,
    ) -> Self {
        Self {
            port,
            embedder_proxy,
            resource_threads,
            constellation_sender,
        }
    }

    fn run(&self, _receiver: UnboundedReceiver<WebDriverMsg>) {
        let _address = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), self.port));
        tokio::runtime::LocalRuntime::new()
            .expect("Runtime creation failed")
            .block_on(async move {
                // let remote_end_state = &self.remote_end_state;
                // let embedder_proxy = &self.embedder_proxy;
                // let resource_threads = &self.resource_threads;
                // let constellation_sender = &self.constellation_sender;

                // let (_, sender) = SessionOldOwning::start_static(
                //     remote_end_state.clone(),
                //     embedder_proxy.clone(),
                //     resource_threads.clone(),
                //     constellation_sender.clone(),
                // );
                // Listener::start(address, remote_end_state.clone(), sender);

                // let forward = Self::handle_thread_message(remote_end_state.clone(), receiver);

                // forward.await
                todo!()
            });
    }

    // /// Handle thread messages from constellation/script/...
    //     async fn handle_thread_message(
    //         remote_end_state: Rc<RemoteEndState>,
    //         mut receiver: UnboundedReceiver<WebDriverMsg>,
    //     ) {
    //         while let Some(msg) = receiver.recv().await {
    //             match msg {
    //                 WebDriverMsg::FromConstellation(constellation_to_web_driver_message) => todo!(),
    //                 WebDriverMsg::FromScript(msg) => match msg {
    //                     ScriptToWebDriverMsg::LogEntryAdded(items, entry_added) => todo!(),
    //                     ScriptToWebDriverMsg::RealmCreated(
    //                         (browsing_context_id, pipeline_id, worker_id, webview_id),
    //                         generic_sender,
    //                     ) => {
    //                         // realm
    //                         remote_end_state.realms.write().await.insert(
    //                             RealmId(pipeline_id, worker_id),
    //                             // TODO: faked, replace with true info
    //                             RealmInfo::WindowRealmInfo(WindowRealmInfo {
    //                                 r#type: WindowRealmInfoType::Window,
    //                                 base_realm_info: BaseRealmInfo {
    //                                     realm: "".to_string(),
    //                                     origin: "".to_string(),
    //                                 },
    //                                 context: browsing_context_id.to_string(),
    //                                 user_context: None,
    //                                 sandbox: None,
    //                             }),
    //                         );

    //                         remote_end_state.navigables.write().await.insert(
    //                             browsing_context_id,
    //                             Navigable {
    //                                 id: browsing_context_id,
    //                                 // unknown here
    //                                 original_opener: None,
    //                                 sender: generic_sender.clone(),
    //                                 webview_id: Some(webview_id),
    //                                 active_document: pipeline_id,
    //                             },
    //                         );
    //                     },
    //                     ScriptToWebDriverMsg::ChannelMessage { channel, data } => todo!(),
    //                     ScriptToWebDriverMsg::FileDialogOpened(file_dialog_opened) => todo!(),
    //                 },
    //             }

    //             // TODO: should not directly forward
    //             // forward to each session
    //             // let msg = Rc::new(msg);
    //             // for session in remote_end_state.active_sessions.read().await.values() {
    //             //     if let Err(e) = session.sender.send(SessionMessage::WebDriver(msg.clone())) {
    //             //         log::warn!("Sending constellation message to session failed: {e:?}");
    //             //     }
    //             // }
    //         }
    //     }
}
