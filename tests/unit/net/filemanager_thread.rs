/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use net::filemanager_thread::{FileManagerThreadFactory, UIProvider};
use net_traits::blob_url_store::BlobURLStoreError;
use net_traits::filemanager_thread::{FilterPattern, FileManagerThreadMsg, FileManagerThreadError, ReadFileProgress};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

const TEST_PROVIDER: &'static TestProvider = &TestProvider;

struct TestProvider;

impl UIProvider for TestProvider {
    fn open_file_dialog(&self, _path: &str, _patterns: Vec<FilterPattern>) -> Option<String> {
        Some("test.jpeg".to_string())
    }

    fn open_file_dialog_multi(&self, _path: &str, _patterns: Vec<FilterPattern>) -> Option<Vec<String>> {
        Some(vec!["test.jpeg".to_string()])
    }
}

#[test]
fn test_filemanager() {
    let chan: IpcSender<FileManagerThreadMsg> = FileManagerThreadFactory::new(TEST_PROVIDER);

    // Try to open a dummy file "tests/unit/net/test.jpeg" in tree
    let mut handler = File::open("test.jpeg").expect("test.jpeg is stolen");
    let mut test_file_content = vec![];

    handler.read_to_end(&mut test_file_content)
           .expect("Read tests/unit/net/test.jpeg error");

    let patterns = vec![FilterPattern(".txt".to_string())];
    let origin = "test.com".to_string();

    {
        // Try to select a dummy file "tests/unit/net/test.jpeg"
        let (tx, rx) = ipc::channel().unwrap();
        chan.send(FileManagerThreadMsg::SelectFile(patterns.clone(), tx, origin.clone(), None)).unwrap();
        let selected = rx.recv().expect("Broken channel")
                                .expect("The file manager failed to find test.jpeg");

        // Expecting attributes conforming the spec
        assert_eq!(selected.filename, PathBuf::from("test.jpeg"));
        assert_eq!(selected.type_string, "image/jpeg".to_string());

        // Test by reading, expecting same content
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            chan.send(FileManagerThreadMsg::ReadFile(tx2, selected.id.clone(), false, origin.clone())).unwrap();

            let msg = rx2.recv().expect("Broken channel");

            if let ReadFileProgress::Meta(blob_buf) = msg.expect("File manager reading failure is unexpected") {
                let mut bytes = blob_buf.bytes;

                loop {
                    match rx2.recv().expect("Broken channel").expect("File manager reading failure is unexpected") {
                        ReadFileProgress::Meta(_) => {
                            panic!("Invalid FileManager reply");
                        }
                        ReadFileProgress::Partial(mut bytes_in) => {
                            bytes.append(&mut bytes_in);
                        }
                        ReadFileProgress::EOF => {
                            break;
                        }
                    }
                }

                assert_eq!(test_file_content, bytes, "Read content differs");
            } else {
                panic!("Invalid FileManager reply");
            }
        }

        // Delete the id
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            chan.send(FileManagerThreadMsg::DecRef(selected.id.clone(), origin.clone(), tx2)).unwrap();

            let ret = rx2.recv().expect("Broken channel");
            assert!(ret.is_ok(), "DecRef is not okay");
        }

        // Test by reading again, expecting read error because we invalidated the id
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            chan.send(FileManagerThreadMsg::ReadFile(tx2, selected.id.clone(), false, origin.clone())).unwrap();

            let msg = rx2.recv().expect("Broken channel");

            match msg {
                Err(FileManagerThreadError::BlobURLStoreError(BlobURLStoreError::InvalidFileID)) => {},
                other => {
                    assert!(false, "Get unexpected response after deleting the id: {:?}", other);
                }
            }
        }
    }

    let _ = chan.send(FileManagerThreadMsg::Exit);

    {
        let (tx, rx) = ipc::channel().unwrap();
        let _ = chan.send(FileManagerThreadMsg::SelectFile(patterns.clone(), tx, origin.clone(), None));

        assert!(rx.try_recv().is_err(), "The thread should not respond normally after exited");
    }
}
