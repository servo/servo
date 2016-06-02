/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use net::filemanager_thread::FileManagerThreadFactory;
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerThreadError};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[test]
fn test_filemanager() {
    let chan: IpcSender<FileManagerThreadMsg> = FileManagerThreadFactory::new();

    let opened = File::open("test.txt");
    let mut test_file_content = vec![];
    let opened_ok = opened.is_ok();

    if let Ok(mut handler) = opened {
        handler.read_to_end(&mut test_file_content)
               .expect("Read tests/unit/net/test.txt error");
    }

    {
        // Try to select a dummy file "tests/unit/net/test.txt"
        let (tx, rx) = ipc::channel().unwrap();
        chan.send(FileManagerThreadMsg::SelectFile(tx)).unwrap();
        let msg = rx.recv().expect("File manager channel is broken");

        if !msg.is_ok() && opened_ok {
            assert!(false, "The file manager failed to find test.txt");
        } else if msg.is_ok() && !opened_ok {
            assert!(false, "The file manager found a test.txt which doesn't exist!");
        } else if msg.is_ok() && opened_ok {
            // Expecting attributes conforming the spec
            let selected = msg.unwrap();
            assert!(selected.filename == PathBuf::from("test.txt"));
            assert!(selected.type_string == "text/plain".to_string());

            // Test by reading, expecting same content
            {
                let (tx2, rx2) = ipc::channel().unwrap();
                chan.send(FileManagerThreadMsg::ReadFile(tx2, selected.id.clone())).unwrap();

                let msg = rx2.recv().expect("File manager channel is broken");

                if let Ok(vec) = msg {
                    assert!(test_file_content == vec, "Read content differs");
                } else {
                    assert!(false, "File manager reading failure is unexpected {:?}", msg);
                }
            }

            // Delete the id
            chan.send(FileManagerThreadMsg::DeleteFileID(selected.id.clone())).unwrap();

            // Test by reading again, expecting read error because we invalidated the id
            {
                let (tx2, rx2) = ipc::channel().unwrap();
                chan.send(FileManagerThreadMsg::ReadFile(tx2, selected.id.clone())).unwrap();

                let msg = rx2.recv().expect("File manager channel is broken");

                match msg {
                    Err(FileManagerThreadError::ReadFileError) => {},
                    other => {
                        assert!(false, "Get unexpected response after deleting the id: {:?}", other);
                    }
                }
            }

        } else {
            assert!(false, "test.txt is stolen");
        }
    }

    let _ = chan.send(FileManagerThreadMsg::Exit);

    {
        let (tx, rx) = ipc::channel().unwrap();
        let _ = chan.send(FileManagerThreadMsg::SelectFile(tx));

        match rx.try_recv() {
            Ok(_) => assert!(false, "The thread should not respond normally after exited"),
            Err(_) => {},
        }
    }
}
