/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc;
use net::resource_task::new_resource_task;
use net_traits::hosts::{parse_hostsfile, host_replacement};
use net_traits::{ControlMsg, LoadData, LoadConsumer, ProgressMsg};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use url::Url;

#[test]
fn test_exit() {
    let resource_task = new_resource_task("".to_owned(), None);
    resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn test_bad_scheme() {
    let resource_task = new_resource_task("".to_owned(), None);
    let (start_chan, start) = ipc::channel().unwrap();
    let url = url!("bogus://whatever");
    resource_task.send(ControlMsg::Load(LoadData::new(url, None), LoadConsumer::Channel(start_chan), None)).unwrap();
    let response = start.recv().unwrap();
    match response.progress_port.recv().unwrap() {
      ProgressMsg::Done(result) => { assert!(result.is_err()) }
      _ => panic!("bleh")
    }
    resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn test_parse_hostsfile() {
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com\n127.0.0.2 servo.test.server";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_parse_malformed_hostsfile() {
    let mock_hosts_file_content = "malformed file\n127.0.0.1 foo.bar.com\nservo.test.server 127.0.0.1";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(1, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_line_comment() {
    let mock_hosts_file_content = "# this is a line comment\n127.0.0.1 foo.bar.com\n# anothercomment";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(1, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_end_of_line_comment() {
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com # line ending comment\n127.0.0.2 servo.test.server #comment";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_2_hostnames_for_1_address() {
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com baz.bar.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"baz.bar.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_4_hostnames_for_1_address() {
    let mock_hosts_file_content = "127.0.0.1 moz.foo.com moz.bar.com moz.baz.com moz.moz.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(4, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.foo.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.baz.com".to_owned()).unwrap());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"moz.moz.com".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_tabs_instead_spaces() {
    let mock_hosts_file_content = "127.0.0.1\tfoo.bar.com\n127.0.0.2\tservo.test.server";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(2, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_parse_hostsfile_with_valid_ipv4_addresses()
{
    let mock_hosts_file_content =
        "255.255.255.255 foo.bar.com\n169.0.1.201 servo.test.server\n192.168.5.0 servo.foo.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(3, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_invalid_ipv4_addresses()
{
    let mock_hosts_file_content = "256.255.255.255 foo.bar.com\n169.0.1000.201 servo.test.server \
                                   \n192.168.5.500 servo.foo.com\n192.abc.100.2 test.servo.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(0, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_valid_ipv6_addresses()
{
    let mock_hosts_file_content = "2001:0db8:0000:0000:0000:ff00:0042:8329 foo.bar.com\n\
                                   2001:db8:0:0:0:ff00:42:8329 moz.foo.com\n\
                                   2001:db8::ff00:42:8329 foo.moz.com moz.moz.com\n\
                                   0000:0000:0000:0000:0000:0000:0000:0001 bar.moz.com\n\
                                   ::1 foo.bar.baz baz.foo.com\n\
                                   2001:0DB8:85A3:0042:1000:8A2E:0370:7334 baz.bar.moz\n\
                                   2002:0DB8:85A3:0042:1000:8A2E:0370:7334/96 baz2.bar.moz\n\
                                   2002:0DB8:85A3:0042:1000:8A2E:0370:7334/128 baz3.bar.moz\n\
                                   :: unspecified.moz.com\n\
                                   ::/128 unspecified.address.com";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(12, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_invalid_ipv6_addresses()
{
    let mock_hosts_file_content = "12001:0db8:0000:0000:0000:ff00:0042:8329 foo.bar.com\n\
                                   2001:zdb8:0:0:0:gg00:42:t329 moz.foo.com\n\
                                   2001:db8::ff00:42:8329:1111:1111:42 foo.moz.com moz.moz.com\n\
                                   2002:0DB8:85A3:0042:1000:8A2E:0370:7334/1289 baz3.bar.moz";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(0, (*hosts_table).len());
}

#[test]
fn test_parse_hostsfile_with_end_of_line_whitespace()
{
    let mock_hosts_file_content = "127.0.0.1 foo.bar.com \n\
                                   2001:db8:0:0:0:ff00:42:8329 moz.foo.com\n \
                                   127.0.0.2 servo.test.server ";
    let hosts_table = parse_hostsfile(mock_hosts_file_content);
    assert_eq!(3, (*hosts_table).len());
    assert_eq!("127.0.0.1".to_owned(), *hosts_table.get(&"foo.bar.com".to_owned()).unwrap());
    assert_eq!("2001:db8:0:0:0:ff00:42:8329".to_owned(), *hosts_table.get(&"moz.foo.com".to_owned()).unwrap());
    assert_eq!("127.0.0.2".to_owned(), *hosts_table.get(&"servo.test.server".to_owned()).unwrap());
}

#[test]
fn test_replace_hosts() {
    let mut host_table_box = Box::new(HashMap::new());
    host_table_box.insert("foo.bar.com".to_owned(), "127.0.0.1".to_owned());
    host_table_box.insert("servo.test.server".to_owned(), "127.0.0.2".to_owned());

    let host_table: *mut HashMap<String, String> = Box::into_raw(host_table_box);

    let url = url!("http://foo.bar.com:8000/foo");
    assert_eq!(host_replacement(host_table, &url).domain().unwrap(), "127.0.0.1");

    let url = url!("http://servo.test.server");
    assert_eq!(host_replacement(host_table, &url).domain().unwrap(), "127.0.0.2");

    let url = url!("http://a.foo.bar.com");
    assert_eq!(host_replacement(host_table, &url).domain().unwrap(), "a.foo.bar.com");
}

#[test]
fn test_cancelled_listener() {
    use std::io::Write;
    use std::net::TcpListener;
    use std::thread;

    // http_loader always checks for headers in the response
    let header = vec!["HTTP/1.1 200 OK",
                      "Server: test-server",
                      "Content-Type: text/plain",
                      "\r\n"];
    let body = vec!["Yay!", "We're doomed!"];

    // Setup a TCP server to which requests are made
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (body_sender, body_receiver) = channel();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            // immediately stream the headers once the connection has been established
            let _ = stream.write(header.join("\r\n").as_bytes());
            // wait for the main thread to send the body, so as to ensure that we're
            // doing everything sequentially
            let body_vec: Vec<&str> = body_receiver.recv().unwrap();
            let _ = stream.write(body_vec.join("\r\n").as_bytes());
        }
    });

    let resource_task = new_resource_task("".to_owned(), None);
    let (sender, receiver) = ipc::channel().unwrap();
    let (id_sender, id_receiver) = ipc::channel().unwrap();
    let (sync_sender, sync_receiver) = ipc::channel().unwrap();
    let url = Url::parse(&format!("http://127.0.0.1:{}", port)).unwrap();

    resource_task.send(ControlMsg::Load(LoadData::new(url, None),
                                        LoadConsumer::Channel(sender),
                                        Some(id_sender))).unwrap();
    // get the `ResourceId` and send a cancel message, which should stop the loading loop
    let res_id = id_receiver.recv().unwrap();
    resource_task.send(ControlMsg::Cancel(res_id)).unwrap();
    // synchronize with the resource_task loop, so that we don't simply send everything at once!
    resource_task.send(ControlMsg::Synchronize(sync_sender)).unwrap();
    let _ = sync_receiver.recv();
    // now, let's send the body, because the connection is still active and data would be loaded
    // (but, the loading has been cancelled)
    let _ = body_sender.send(body);
    let response = receiver.recv().unwrap();
    match response.progress_port.recv().unwrap() {
        ProgressMsg::Done(result) => assert_eq!(result.unwrap_err(), "load cancelled".to_owned()),
        _ => panic!("baaaah!"),
    }
    resource_task.send(ControlMsg::Exit).unwrap();
}
