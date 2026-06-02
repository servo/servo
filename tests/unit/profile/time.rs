/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::thread;

use ::time::Duration;
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
        Duration::seconds_f64(1.234),
        Duration::seconds_f64(3.246),
        Duration::seconds_f64(3.546),
        Duration::seconds_f64(5.000),
        Duration::seconds_f64(5.324),
        Duration::seconds_f64(7.345),
        Duration::seconds_f64(9.235),
        Duration::seconds_f64(10.234),
        Duration::seconds_f64(13.250),
        Duration::seconds_f64(15.000),
    ];
    let (even_mean, even_median, even_min, even_max) = time::Profiler::get_statistics(&even_data);

    assert_eq!(7341, even_mean.whole_milliseconds());
    assert_eq!(Duration::seconds_f64(7.345), even_median);
    assert_eq!(Duration::seconds_f64(1.234), even_min);
    assert_eq!(Duration::seconds_f64(15.000), even_max);

    let odd_data = vec![
        Duration::seconds_f64(1.234),
        Duration::seconds_f64(3.246),
        Duration::seconds_f64(3.546),
        Duration::seconds_f64(5.000),
        Duration::seconds_f64(5.324),
        Duration::seconds_f64(7.345),
        Duration::seconds_f64(9.235),
        Duration::seconds_f64(10.234),
        Duration::seconds_f64(13.250),
    ];
    let (odd_mean, odd_median, odd_min, odd_max) = time::Profiler::get_statistics(&odd_data);

    assert_eq!(6490, odd_mean.whole_milliseconds());
    assert_eq!(Duration::seconds_f64(5.324), odd_median);
    assert_eq!(Duration::seconds_f64(1.234), odd_min);
    assert_eq!(Duration::seconds_f64(13.250), odd_max);
}

#[test]
fn channel_profiler_test() {
    let chan = time::Profiler::create(&Some(OutputOptions::Stdout(5.0)), None);
    let (profiled_sender, profiled_receiver) = ProfiledIpc::channel(chan.clone()).unwrap();
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(2));
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
        ProfilerData::Record(time_data) => assert!(time_data[0] > Duration::milliseconds(1500)),
        ProfilerData::NoRecords => assert!(false),
    };
}

#[test]
fn bytes_channel_profiler_test() {
    let chan = time::Profiler::create(&Some(OutputOptions::Stdout(5.0)), None);
    let (profiled_sender, profiled_receiver) = ProfiledIpc::bytes_channel(chan.clone()).unwrap();
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(2));
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
        ProfilerData::Record(time_data) => assert!(time_data[0] > Duration::milliseconds(1500)),
        ProfilerData::NoRecords => assert!(false),
    };
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn time_profiler_unsorted_stats_test() {
    let unsorted_data = vec![
        Duration::microseconds(5000),
        Duration::microseconds(7500),
        Duration::microseconds(1000),
        Duration::microseconds(890),
    ];
    time::Profiler::get_statistics(&unsorted_data);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn time_profiler_data_len_zero() {
    let zero_data = vec![];
    time::Profiler::get_statistics(&zero_data);
}
