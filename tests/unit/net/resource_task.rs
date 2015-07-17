/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::resource_task::{new_resource_task, parse_hostsfile, replace_hosts};
use net_traits::{ControlMsg, LoadData, LoadConsumer};
use net_traits::ProgressMsg;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use url::Url;


#[test]
fn test_exit() {
    let resource_task = new_resource_task(None, None);
    resource_task.send(ControlMsg::Exit).unwrap();
}

#[test]
fn test_bad_scheme() {
    let resource_task = new_resource_task(None, None);
    let (start_chan, start) = channel();
    let url = Url::parse("bogus://whatever").unwrap();
    resource_task.send(ControlMsg::Load(LoadData::new(url, None), LoadConsumer::Channel(start_chan))).unwrap();
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
    use std::net::TcpListener;

    let mut host_table_box = Box::new(HashMap::new());
    host_table_box.insert("foo.bar.com".to_owned(), "127.0.0.1".to_owned());
    host_table_box.insert("servo.test.server".to_owned(), "127.0.0.2".to_owned());

    let host_table: *mut HashMap<String, String> = Box::into_raw(host_table_box);

    //Start the TCP server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    //Start the resource task and make a request to our TCP server
    let resource_task = new_resource_task(None, None);
    let (start_chan, _) = channel();
    let url = Url::parse(&format!("http://foo.bar.com:{}", port)).unwrap();
    let msg = ControlMsg::Load(replace_hosts(LoadData::new(url, None), host_table),
                               LoadConsumer::Channel(start_chan));
    resource_task.send(msg).unwrap();

    match listener.accept() {
        Ok(..) => assert!(true, "received request"),
        Err(_) => assert!(false, "error")
    }

    resource_task.send(ControlMsg::Exit).unwrap();
}
