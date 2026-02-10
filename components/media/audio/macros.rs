/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_export]
macro_rules! make_message_handler(
    (
        $(
            $node:ident: $handler:ident
         ),+
    ) => (
        fn message_specific(&mut self, msg: $crate::node::AudioNodeMessage, sample_rate: f32) {
            match msg {
                $($crate::node::AudioNodeMessage::$node(m) => self.$handler(m, sample_rate)),+,
                _ => (),
            }
        }
    );
);

#[macro_export]
macro_rules! make_state_change(
    ($fn_name:ident, $state:ident, $render_msg:ident) => (
        pub fn $fn_name(&self) -> StateChangeResult {
            self.state.set(ProcessingState::$state);
            let (tx, rx) = mpsc::channel();
            let _ = self.sender.send(AudioRenderThreadMsg::$render_msg(tx));
            rx.recv().unwrap()
        }
    );
);

#[macro_export]
macro_rules! make_render_thread_state_change(
    ($fn_name:ident, $state:ident, $sink_method:ident) => (
        fn $fn_name(&mut self) -> StateChangeResult {
            if self.state == ProcessingState::$state {
                return Some(());
            }
            self.state = ProcessingState::$state;
            self.sink.$sink_method().ok()
        }
    );
);
