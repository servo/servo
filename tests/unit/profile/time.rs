/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use profile::time;
use profile_traits::time::ProfilerMsg;
use ipc_channel::ipc;

#[test]
fn time_profiler_smoke_test() {
    let chan = time::Profiler::create(&None, None);
    assert!(true, "Can create the profiler thread");

    let (ipcchan, ipcport) = ipc::channel().unwrap();
    chan.send(ProfilerMsg::Exit(ipcchan));
    assert!(true, "Can tell the profiler thread to exit");
}
