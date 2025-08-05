use std::thread;

use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use net_traits::speech_thread::SpeechSynthesisThreadMsg;

pub trait SpeechSynthesisThreadFactory {
    fn new() -> Self;
}

impl SpeechSynthesisThreadFactory for IpcSender<SpeechSynthesisThreadMsg> {
    fn new() -> Self {
        let (chan, port) = ipc::channel().unwrap();
        thread::Builder::new()
            .name("SpeechSynthesisManager".to_owned())
            .spawn(move || {
                SpeechSynthesisManager::new(port).start();
            })
            .expect("Thread spawning failed");
        chan
    }
}

struct SpeechSynthesisManager {
    port: ipc::IpcReceiver<SpeechSynthesisThreadMsg>,
}

impl SpeechSynthesisManager {
    fn new(port: ipc::IpcReceiver<SpeechSynthesisThreadMsg>) -> Self {
        SpeechSynthesisManager { port }
    }

    fn start(self) {
        while let Ok(msg) = self.port.recv() {
            match msg {
                SpeechSynthesisThreadMsg::Speak(text) => {},
                SpeechSynthesisThreadMsg::Cancel => {},
                SpeechSynthesisThreadMsg::Pause => {},
                SpeechSynthesisThreadMsg::Resume => {},
                SpeechSynthesisThreadMsg::GetVoices(sender) => {
                    let _ = sender.send(vec![]);
                },
            }
        }
    }
}
