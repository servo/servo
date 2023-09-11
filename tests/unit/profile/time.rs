/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::thread;
use std::time::Duration;

use ipc_channel::ipc;
use profile::time;
use profile_traits::ipc as ProfiledIpc;
use profile_traits::time::{ProfilerCategory, ProfilerData, ProfilerMsg};
use servo_config::opts::OutputOptions;

#[test]
fn time_profiler_smoke_test() {
    let chan = time::Profiler::create(&None, None);
    assert!(true, "Can create the profiler thread");

    let (ipcchan, _ipcport) = ipc::channel().unwrap();
    chan.send(ProfilerMsg::Exit(ipcchan));
    assert!(true, "Can tell the profiler thread to exit");
}

#[test]
fn time_profiler_stats_test() {
    let even_data = vec![
        1.234, 3.24567, 3.54578, 5.0, 5.324, 7.345, 9.2345, 10.2342345, 13.2599, 15.0,
    ];
    let (even_mean, even_median, even_min, even_max) = time::Profiler::get_statistics(&even_data);

    assert_eq!(7.34230845, even_mean);
    assert_eq!(7.345, even_median);
    assert_eq!(1.234, even_min);
    assert_eq!(15.0, even_max);

    let odd_data = vec![
        1.234, 3.24567, 3.54578, 5.0, 5.324, 7.345, 9.2345, 10.2342345, 13.2599,
    ];
    let (odd_mean, odd_median, odd_min, odd_max) = time::Profiler::get_statistics(&odd_data);

    assert_eq!(6.491453833333334, odd_mean);
    assert_eq!(5.324, odd_median);
    assert_eq!(1.234, odd_min);
    assert_eq!(13.2599, odd_max);
}

#[test]
fn channel_profiler_test() {
    let chan = time::Profiler::create(&Some(OutputOptions::Stdout(5.0)), None);
    let (profiled_sender, profiled_receiver) = ProfiledIpc::channel(chan.clone()).unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        profiled_sender.send(43).unwrap();
    });

    let val_profile_receiver = profiled_receiver.recv().unwrap();
    assert_eq!(val_profile_receiver, 43);

    let (sender, receiver) = ipc::channel().unwrap();
    chan.send(ProfilerMsg::Get(
        (ProfilerCategory::IpcReceiver, None),
        sender.clone(),
    ));

    match receiver.recv().unwrap() {
        // asserts that the time spent in the sleeping thread is more than 1500 milliseconds
        ProfilerData::Record(time_data) => assert!(time_data[0] > 1.5e3),
        ProfilerData::NoRecords => assert!(false),
    };
}

#[test]
fn bytes_channel_profiler_test() {
    let chan = time::Profiler::create(&Some(OutputOptions::Stdout(5.0)), None);
    let (profiled_sender, profiled_receiver) = ProfiledIpc::bytes_channel(chan.clone()).unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        profiled_sender.send(&[1, 2, 3]).unwrap();
    });

    let val_profile_receiver = profiled_receiver.recv().unwrap();
    assert_eq!(val_profile_receiver, [1, 2, 3]);

    let (sender, receiver) = ipc::channel().unwrap();
    chan.send(ProfilerMsg::Get(
        (ProfilerCategory::IpcBytesReceiver, None),
        sender.clone(),
    ));

    match receiver.recv().unwrap() {
        // asserts that the time spent in the sleeping thread is more than 1500 milliseconds
        ProfilerData::Record(time_data) => assert!(time_data[0] > 1.5e3),
        ProfilerData::NoRecords => assert!(false),
    };
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn time_profiler_unsorted_stats_test() {
    let unsorted_data = vec![5.0, 7.5, 1.0, 8.9];
    time::Profiler::get_statistics(&unsorted_data);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn time_profiler_data_len_zero() {
    let zero_data = vec![];
    time::Profiler::get_statistics(&zero_data);
}
