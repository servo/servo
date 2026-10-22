cfg_select! {
    target_os = "linux" => {
        pub type NetworkMonitor = linux::NetworkMonitor;
    },
    target_os = "windows" => {
        pub type NetworkMonitor = windows::NetworkMonitor;
    },
    // TODO: macos depends on an unreleased objc2 version (objc2-network)
    _ => {
        pub type NetworkMonitor = dummy::NetworkMonitor;
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod dummy {
    use winit::event_loop::EventLoopProxy;

    use crate::desktop::event_loop::AppEvent;

    pub(crate) struct NetworkMonitor;

    impl NetworkMonitor {
        pub(crate) fn new(_event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
            Self
        }
    }
}

/// Monitor active network links via `rtnetlink`, using an initial dump followed
/// by change notifications. Linux device is online if there is at least one active link
/// that is not loopback or tun,
#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;
    use std::thread;

    use futures_util::StreamExt;
    use log::error;
    use rtnetlink::constants::RTMGRP_LINK;
    use rtnetlink::new_connection;
    use rtnetlink::packet_core::{NLM_F_DUMP, NLM_F_REQUEST, NetlinkMessage, NetlinkPayload};
    use rtnetlink::packet_route::RouteNetlinkMessage;
    use rtnetlink::packet_route::link::{LinkAttribute, LinkFlags, LinkHeader, LinkMessage};
    use rtnetlink::sys::{AsyncSocket, SocketAddr};
    use tokio::sync::mpsc::{self, Receiver, Sender};
    use winit::event_loop::EventLoopProxy;

    use crate::desktop::event_loop::AppEvent;

    pub(crate) struct NetworkMonitor {
        shutdown_sender: Sender<()>,
    }

    impl NetworkMonitor {
        pub(crate) fn new(event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
            let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);
            thread::Builder::new()
                .name("network-monitor".into())
                .spawn(move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("failed to build tokio runtime");
                    runtime.block_on(watch_rtnetlink(event_loop_proxy, shutdown_receiver))
                })
                .expect("failed to spawn network monitor thread");
            Self { shutdown_sender }
        }
    }

    impl Drop for NetworkMonitor {
        fn drop(&mut self) {
            if let Err(err) = self.shutdown_sender.blocking_send(()) {
                error!("failed to shutdown network monitor {:?}", err);
            }
        }
    }

    async fn watch_rtnetlink(
        event_loop_proxy: EventLoopProxy<AppEvent>,
        mut shutdown_receiver: Receiver<()>,
    ) {
        let mut links = HashMap::new();
        let mut previous_online = true;

        let update_online = |online_links: &HashMap<u32, Option<String>>, previous: &mut bool| {
            let current = online_links
                .values()
                .any(|name| !name.as_ref().is_some_and(|name| name.starts_with("tun")));

            if current != *previous {
                *previous = current;

                let _ = event_loop_proxy.send_event(AppEvent::SetNetworkOnlineState(current));
            }
        };

        let Ok((mut connection, mut handle, mut messages)) = new_connection() else {
            error!("failed to connection to rtnetlink");
            return;
        };

        let addr = SocketAddr::new(0, RTMGRP_LINK);
        let Ok(_) = connection.socket_mut().socket_mut().bind(&addr) else {
            error!("failed to register rtnetlink broadcast");
            return;
        };
        tokio::spawn(connection);

        // initial dump
        let mut msg = NetlinkMessage::from(RouteNetlinkMessage::GetLink(LinkMessage::default()));
        msg.header.flags = NLM_F_REQUEST | NLM_F_DUMP;

        if let Ok(mut stream) = handle.request(msg) {
            while let Some(msg) = stream.next().await {
                handle_rtnetlink_message(msg, &mut links);
            }
            update_online(&links, &mut previous_online);
        }

        // listen to link change notifications
        loop {
            tokio::select! {
                _ = shutdown_receiver.recv() => return,
                Ok((msg, _)) = messages.recv() => {
                    handle_rtnetlink_message(msg, &mut links);
                    update_online(&links, &mut previous_online);
                },
            };
        }
    }

    fn handle_rtnetlink_message(
        msg: NetlinkMessage<RouteNetlinkMessage>,
        online_links: &mut HashMap<u32, Option<String>>,
    ) {
        match msg.payload {
            NetlinkPayload::Done(_done_message) => {},
            NetlinkPayload::Error(_error_message) => {},
            NetlinkPayload::InnerMessage(inner) => match inner {
                RouteNetlinkMessage::NewLink(msg) => {
                    let LinkHeader { flags, index, .. } = msg.header;
                    if !flags.contains(LinkFlags::Loopback) &&
                        flags.contains(LinkFlags::Up | LinkFlags::LowerUp | LinkFlags::Running)
                    {
                        let name = msg.attributes.into_iter().find_map(|attr| match attr {
                            LinkAttribute::IfName(name) => Some(name),
                            _ => None,
                        });
                        online_links.insert(index, name);
                    } else {
                        online_links.remove(&index);
                    }
                },
                RouteNetlinkMessage::DelLink(msg) => {
                    online_links.remove(&msg.header.index);
                },
                _ => {},
            },
            _ => {},
        }
    }
}

/// Monitor active network links via `NotifyUnicastI[AddressChange` and `GetNetworkConnectivityHint`
#[cfg(target_os = "windows")]
mod windows {
    use std::ffi::c_void;
    use std::thread;
    use std::time::Duration;

    use crossbeam_channel::{self, Receiver, Sender, select_biased};
    use log::error;
    use windows::Win32::Foundation::{HANDLE, NO_ERROR};
    use windows::Win32::NetworkManagement::IpHelper::{
        CancelMibChangeNotify2, GetNetworkConnectivityHint, MIB_NOTIFICATION_TYPE,
        MIB_UNICASTIPADDRESS_ROW, NotifyUnicastIpAddressChange,
    };
    use windows::Win32::Networking::WinSock::{
        AF_UNSPEC, NL_NETWORK_CONNECTIVITY_HINT, NetworkConnectivityLevelHintNone,
    };
    use winit::event_loop::EventLoopProxy;

    use crate::desktop::event_loop::AppEvent;

    const MAX_POLL: usize = 20;
    const POLL_INTERVAL: Duration = Duration::from_secs(1);

    pub(crate) struct NetworkMonitor {
        shutdown_sender: Sender<()>,
    }

    impl NetworkMonitor {
        pub(crate) fn new(event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
            let (shutdown_sender, shutdown_receiver) = crossbeam_channel::bounded(1);

            thread::Builder::new()
                .name("network-monitor".into())
                .spawn(move || {
                    watch_connectivity(event_loop_proxy, shutdown_receiver);
                })
                .expect("failed to spawn network monitor thread");

            Self { shutdown_sender }
        }
    }

    impl Drop for NetworkMonitor {
        fn drop(&mut self) {
            let _ = self.shutdown_sender.send(());
        }
    }

    fn watch_connectivity(
        event_loop_proxy: EventLoopProxy<AppEvent>,
        shutdown_receiver: Receiver<()>,
    ) {
        let (address_change_sender, address_change_receiver) = crossbeam_channel::bounded::<()>(1);
        let mut handle = HANDLE::default();

        let callback_context = Box::new(address_change_sender);
        let callback_context_ptr = &*callback_context as *const Sender<()> as *const c_void;

        // register address change notification
        let result = unsafe {
            NotifyUnicastIpAddressChange(
                AF_UNSPEC,
                Some(on_address_change),
                Some(callback_context_ptr),
                true,
                &mut handle,
            )
        };

        if result != NO_ERROR {
            error!("failed to register address change notifications: {result}");
            return;
        }

        let mut previous_online = true;

        'monitor: loop {
            select_biased! {
                recv(shutdown_receiver) -> _ => break 'monitor,
                recv(address_change_receiver) -> result => {
                    if result.is_err() {
                        break 'monitor;
                    }
                }
            }

            // Since there is a delay between address change notification and `GetNetworkConnectivityHint`,
            // we need to poll 20 times to check if the offline result is stable.
            let mut poll_index = 0;
            loop {
                if let Some(current) = get_current_online() {
                    if current != previous_online {
                        previous_online = current;
                        let _ =
                            event_loop_proxy.send_event(AppEvent::SetNetworkOnlineState(current));
                    }
                }

                if poll_index == MAX_POLL {
                    break;
                }

                select_biased! {
                    recv(shutdown_receiver) -> _ => break 'monitor,
                    recv(address_change_receiver) -> result => {
                        if result.is_err() {
                            break 'monitor;
                        }
                        poll_index = 0;
                    }
                    default(POLL_INTERVAL) => {
                        poll_index += 1;
                    }
                }
            }
        }

        // cancel the notification on exit
        let result = unsafe { CancelMibChangeNotify2(handle) };

        if result == NO_ERROR {
            drop(callback_context);
        } else {
            error!("failed to cancel address change notifications: {result}");
            // if cancellation failed, forget it to avoid use after free.
            std::mem::forget(callback_context);
        }
    }

    extern "system" fn on_address_change(
        context: *const c_void,
        _row: *const MIB_UNICASTIPADDRESS_ROW,
        _notification_type: MIB_NOTIFICATION_TYPE,
    ) {
        let Some(address_change_sender) = (unsafe { (context as *const Sender<()>).as_ref() })
        else {
            return;
        };
        let _ = address_change_sender.try_send(());
    }

    fn get_current_online() -> Option<bool> {
        let mut hint = NL_NETWORK_CONNECTIVITY_HINT::default();
        let result = unsafe { GetNetworkConnectivityHint(&mut hint) };

        if result != NO_ERROR {
            error!("Failed to get network connectivity hint: {result}");
            return None;
        }

        #[expect(non_snake_case)]
        Some(!matches!(
            hint.ConnectivityLevel,
            NetworkConnectivityLevelHintNone
        ))
    }
}
