/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

type NetworkDelegateCallback = Box<dyn Fn(bool) + Send + Sync + 'static>;

enum Message {
    Shutdown,
    SetCallback(NetworkDelegateCallback),
}

/// A delegate that is responsible for system network detection. This currently
/// works for Linux and Windows. The macOS inplementation is blocked by objc2-network
/// crate. A default delegate is automatically used that implements network support.
/// An embedding application can override this delegate by using this trait.
pub trait NetworkDelegate {
    /// A request to setup callback for network delegate.
    ///
    /// Servo will call this when `set_network_delegate` is called.
    fn start(&self, callback: NetworkDelegateCallback);
    /// A request to shutdown the background network tasks.
    fn shutdown(&self);
}

cfg_select! {
    target_os = "linux" => {
        pub type DefaultNetworkDelegate = linux::LinuxNetworkDelegate;
    },
    target_os = "windows" => {
        pub type DefaultNetworkDelegate = windows::WindowsNetworkDelegate;
    },
    _ => {
        pub type DefaultNetworkDelegate = dummy::DummyNetworkDelegate;
    }
}

/// Monitor active network links via netlink api, using an initial dump followed
/// by change notifications. Linux device is online if there is at least one active link
/// that is not loopback or tun,
#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;
    use std::ffi::{CStr, c_char, c_int, c_uchar, c_void};
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::{io, thread};

    use libc::{
        AF_NETLINK, AF_UNSPEC, IFLA_IFNAME, NETLINK_ROUTE, NLM_F_DUMP, NLM_F_REQUEST, NLMSG_ALIGN,
        NLMSG_DATA, NLMSG_LENGTH, NLMSG_NEXT, NLMSG_OK, RTM_DELLINK, RTM_GETLINK, RTM_NEWLINK,
        RTMGRP_LINK, SOCK_CLOEXEC, SOCK_NONBLOCK, SOCK_RAW, bind, ifinfomsg, nlmsgerr, nlmsghdr,
        recv, rtattr, sa_family_t, sendto, sockaddr, sockaddr_nl, socket, socklen_t,
    };
    use log::error;
    use tokio::io::unix::AsyncFd;
    use tokio::sync::mpsc::{self, Receiver, Sender};

    use crate::network_delegate::linux::macros::*;
    use crate::network_delegate::{Message, NetworkDelegate, NetworkDelegateCallback};

    pub struct LinuxNetworkDelegate {
        sender: Sender<Message>,
    }

    impl LinuxNetworkDelegate {
        pub fn new() -> Self {
            let (sender, receiver) = mpsc::channel(8);

            thread::Builder::new()
                .name("network-monitor".into())
                .spawn(move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_io()
                        .build()
                        .expect("failed to build tokio runtime");
                    runtime.block_on(watch_netlink(receiver))
                })
                .expect("failed to spawn network monitor thread");

            Self { sender }
        }
    }

    impl NetworkDelegate for LinuxNetworkDelegate {
        fn start(&self, callback: NetworkDelegateCallback) {
            _ = self.sender.try_send(Message::SetCallback(callback));
        }

        fn shutdown(&self) {
            _ = self.sender.try_send(Message::Shutdown);
        }
    }

    /// C macros that are not provided by `libc`.
    #[expect(non_snake_case)]
    mod macros {
        use std::ffi::{c_int, c_uint, c_void};

        use libc::rtattr;

        pub(super) const IFF_LOOPBACK: c_uint = libc::IFF_LOOPBACK as c_uint;
        pub(super) const IFF_UP: c_uint = libc::IFF_UP as c_uint;
        pub(super) const IFF_LOWER_UP: c_uint = libc::IFF_LOWER_UP as c_uint;
        pub(super) const IFF_RUNNING: c_uint = libc::IFF_RUNNING as c_uint;
        pub(super) const NLMSG_DONE: u16 = libc::NLMSG_DONE as u16;
        pub(super) const NLMSG_ERROR: u16 = libc::NLMSG_ERROR as u16;

        const RTA_ALIGNTO: usize = 4;

        #[inline]
        pub(super) const fn RTA_ALIGN(len: usize) -> usize {
            (len + RTA_ALIGNTO - 1) & !(RTA_ALIGNTO - 1)
        }

        #[inline]
        pub(super) const fn RTA_OK(rta: *const rtattr, len: c_int) -> bool {
            len >= size_of::<rtattr>() as c_int &&
                unsafe { (*rta).rta_len } as usize >= size_of::<rtattr>() &&
                (unsafe { (*rta).rta_len } as c_int) <= len
        }

        #[inline]
        pub(super) const fn RTA_NEXT(rta: *mut rtattr, attrlen: &mut c_int) -> *mut rtattr {
            *attrlen -= RTA_ALIGN(unsafe { (*rta).rta_len } as usize) as c_int;
            unsafe { (rta as *mut u8).add(RTA_ALIGN((*rta).rta_len as usize)) as *mut rtattr }
        }

        #[inline]
        pub(super) const fn RTA_LENGTH(len: usize) -> usize {
            RTA_ALIGN(size_of::<rtattr>()) + len
        }

        #[inline]
        pub(super) const fn RTA_DATA(rta: *const rtattr) -> *mut c_void {
            unsafe { (rta as *mut u8).add(RTA_LENGTH(0)) as *mut c_void }
        }
    }

    fn link_name(ifi: *const ifinfomsg, msg_len: u32) -> Option<String> {
        let mut len = msg_len as c_int - NLMSG_LENGTH(size_of::<ifinfomsg>());

        let mut attr = unsafe { (ifi as *mut u8).add(NLMSG_ALIGN(size_of::<ifinfomsg>()) as usize) }
            as *mut rtattr;

        while RTA_OK(attr, len) {
            if unsafe { (*attr).rta_type } == IFLA_IFNAME {
                let name = unsafe { CStr::from_ptr(RTA_DATA(attr) as *const c_char) };
                return Some(name.to_string_lossy().into_owned());
            }

            attr = RTA_NEXT(attr, &mut len);
        }

        None
    }

    #[repr(C)]
    struct Request {
        nlh: nlmsghdr,
        ifi: ifinfomsg,
    }

    async fn watch_netlink(mut receiver: Receiver<Message>) {
        let mut callback = None;

        let mut online_links: HashMap<i32, Option<String>> = HashMap::new();
        let mut previous_online = true;

        let netlink = match open_netlink()
            // .and_then(|fd| unsafe { AsyncFd::register(fd) }.map_err(Into::into))
            .and_then(|fd| AsyncFd::new(fd))
        {
            Ok(netlink) => netlink,
            Err(err) => {
                error!("failed to connect to rtnetlink: {err}");
                return;
            },
        };

        if let Err(err) = request_dump(netlink.get_ref()) {
            error!("sendto: {err}");
            return;
        }

        let mut is_dump_done = false;
        let mut buf = [0u64; 4096];

        loop {
            tokio::select! {
                biased;

                msg = receiver.recv() => match msg {
                    Some(Message::Shutdown) | None => return,
                    Some(Message::SetCallback(cb)) => callback = Some(cb),
                },

                ready = netlink.readable() => {
                    let mut guard = match ready {
                        Ok(guard) => guard,
                        Err(err) => {
                            error!("readable: {err}");
                            return;
                        }
                    };

                    loop {
                        match guard.try_io(|inner| recv_netlink(inner.get_ref(), &mut buf)) {
                            Ok(Ok(n)) => {
                                if n > 0 {
                                    if !handle_netlink(
                                        &mut buf,
                                        n,
                                        &mut online_links,
                                        &mut is_dump_done,
                                    ) {
                                        return;
                                    }

                                    if is_dump_done && let Some(callback) = &callback {
                                        update_online(
                                            &online_links,
                                            &mut previous_online,
                                            &callback,
                                        );
                                    }
                                }
                            }
                            Ok(Err(err)) => {
                                error!("recv: {err}");
                                return;
                            }
                            Err(_would_block) => break,
                        }
                    }
                }
            }
        }
    }

    fn open_netlink() -> io::Result<OwnedFd> {
        let fd = unsafe {
            socket(
                AF_NETLINK,
                SOCK_RAW | SOCK_CLOEXEC | SOCK_NONBLOCK,
                NETLINK_ROUTE,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let netlink = unsafe { OwnedFd::from_raw_fd(fd) };

        let mut addr: sockaddr_nl = unsafe { MaybeUninit::zeroed().assume_init() };
        addr.nl_family = AF_NETLINK as sa_family_t;
        addr.nl_groups = RTMGRP_LINK as u32;

        if unsafe {
            bind(
                fd,
                &addr as *const _ as *const sockaddr,
                size_of_val(&addr) as socklen_t,
            )
        } < 0
        {
            return Err(io::Error::last_os_error());
        }

        Ok(netlink)
    }

    fn request_dump(netlink: &OwnedFd) -> io::Result<()> {
        let mut nlh: nlmsghdr = unsafe { MaybeUninit::zeroed().assume_init() };
        nlh.nlmsg_len = NLMSG_LENGTH(size_of::<ifinfomsg>()) as u32;
        nlh.nlmsg_type = RTM_GETLINK;
        nlh.nlmsg_flags = (NLM_F_REQUEST | NLM_F_DUMP) as u16;
        nlh.nlmsg_seq = 1;

        let mut ifi: ifinfomsg = unsafe { MaybeUninit::zeroed().assume_init() };
        ifi.ifi_family = AF_UNSPEC as c_uchar;

        let req = Request { nlh, ifi };

        let mut kernel: sockaddr_nl = unsafe { MaybeUninit::zeroed().assume_init() };
        kernel.nl_family = AF_NETLINK as sa_family_t;

        let sent = unsafe {
            sendto(
                netlink.as_raw_fd(),
                &req as *const _ as *const c_void,
                req.nlh.nlmsg_len as usize,
                0,
                &kernel as *const _ as *const sockaddr,
                size_of_val(&kernel) as socklen_t,
            )
        };

        if sent < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    fn recv_netlink(netlink: &OwnedFd, buf: &mut [u64]) -> io::Result<isize> {
        let n = unsafe {
            recv(
                netlink.as_raw_fd(),
                buf.as_mut_ptr() as *mut c_void,
                buf.len() * size_of::<u64>(),
                0,
            )
        };

        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(n)
        }
    }

    fn handle_netlink(
        buf: &mut [u64],
        len: isize,
        online_links: &mut HashMap<i32, Option<String>>,
        is_dump_done: &mut bool,
    ) -> bool {
        let mut remaining = len as c_int;

        let mut p = buf.as_mut_ptr() as *mut nlmsghdr;
        while unsafe { NLMSG_OK(p, remaining) } {
            let next = unsafe { NLMSG_NEXT(p, &mut remaining) };
            let msg_type = unsafe { (*p).nlmsg_type };

            match msg_type {
                NLMSG_DONE => {
                    *is_dump_done = true;
                },
                NLMSG_ERROR => {
                    let error = unsafe { (*(NLMSG_DATA(p) as *const nlmsgerr)).error };
                    if error != 0 {
                        error!("netlink: {}", io::Error::from_raw_os_error(-error));
                        return false;
                    }
                },
                RTM_NEWLINK => {
                    let ifi = unsafe { NLMSG_DATA(p) as *const ifinfomsg };
                    let (index, flags) = unsafe { ((*ifi).ifi_index, (*ifi).ifi_flags) };

                    let loopback = flags & IFF_LOOPBACK != 0;
                    let up = flags & IFF_UP != 0;
                    let lower_up = flags & IFF_LOWER_UP != 0;
                    let running = flags & IFF_RUNNING != 0;

                    if !loopback && up && lower_up && running {
                        online_links.insert(index, link_name(ifi, unsafe { (*p).nlmsg_len }));
                    } else {
                        online_links.remove(&index);
                    }
                },
                RTM_DELLINK => {
                    let index = unsafe { (*(NLMSG_DATA(p) as *const ifinfomsg)).ifi_index };
                    online_links.remove(&index);
                },
                _ => {},
            }

            p = next;
        }

        true
    }

    fn update_online(
        links: &HashMap<i32, Option<String>>,
        previous_online: &mut bool,
        callback: impl Fn(bool),
    ) {
        let current = links
            .values()
            .any(|name| !name.as_ref().is_some_and(|name| name.starts_with("tun")));

        if current != *previous_online {
            *previous_online = current;
            callback(current);
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

    use crate::network_delegate::{Message, NetworkDelegate};

    const MAX_POLL: usize = 20;
    const POLL_INTERVAL: Duration = Duration::from_secs(1);

    pub(crate) struct WindowsNetworkDelegate {
        sender: Sender<Message>,
    }

    impl WindowsNetworkDelegate {
        pub(crate) fn new() -> Self {
            let (sender, receiver) = crossbeam_channel::bounded(8);

            thread::Builder::new()
                .name("network-monitor".into())
                .spawn(move || {
                    watch_connectivity(receiver);
                })
                .expect("failed to spawn network monitor thread");

            Self { sender }
        }
    }

    impl NetworkDelegate for WindowsNetworkDelegate {
        fn start(&self, callback: super::NetworkDelegateCallback) {
            _ = self.sender.send(Message::SetCallback(callback));
        }

        fn shutdown(&self) {
            let _ = self.sender.send(Message::Shutdown);
        }
    }

    fn watch_connectivity(receiver: Receiver<Message>) {
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

        let mut callback = None;
        let mut previous_online = true;

        'monitor: loop {
            select_biased! {
                recv(receiver) -> msg => match msg {
                    Ok(Message::Shutdown) | Err(_) => break 'monitor,
                    Ok(Message::SetCallback(cb)) => callback = Some(cb),
                },
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
                        if let Some(callback) = callback {
                            let _ = callback(current);
                        }
                    }
                }

                if poll_index == MAX_POLL {
                    break;
                }

                select_biased! {
                    recv(receiver) -> msg => match msg {
                        Ok(Message::Shutdown) | Err(_) => break 'monitor,
                        Ok(Message::SetCallback(cb)) => callback = Some(cb),
                    },
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

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod dummy {
    use crate::network_delegate::NetworkDelegate;

    pub(crate) struct DummyNetworkDelegate;

    impl DummyNetworkDelegate {
        pub(crate) fn new() -> Self {
            Self
        }
    }

    impl NetworkDelegate for DummyNetworkDelegate {
        fn start(&self, _callback: super::NetworkDelegateCallback) {}

        fn shutdown(&self) {}
    }
}
