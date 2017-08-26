extern crate futures;
extern crate script_traits;

use futures::sync::mpsc as futures_mpsc;
use script_traits::ConstellationMsg;
use std::sync::mpsc as std_mpsc;

/// Establish a CDP server control channel.
#[inline]
pub fn control_channel() -> (CdpControlSender, CdpControlReceiver) {
    futures_mpsc::unbounded()
}

/// The sender half of a CDP server control channel.
pub type CdpControlSender = futures_mpsc::UnboundedSender<CdpControlMsg>;
/// The receiver half of a CDP server control channel.
pub type CdpControlReceiver = futures_mpsc::UnboundedReceiver<CdpControlMsg>;

/// Inbound control messages to the CDP server.
pub enum CdpControlMsg {
    /// Complete setup with the provided constellation channel handle and start
    /// accepting clients.
    StartAccepting(std_mpsc::Sender<ConstellationMsg>),
}
