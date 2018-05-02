/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use create_embedder_proxy;
use embedder_traits::FilterPattern;
use ipc_channel::ipc;
use net::filemanager_thread::FileManager;
use net_traits::blob_url_store::BlobURLStoreError;
use net_traits::filemanager_thread::{FileManagerThreadMsg, FileManagerThreadError, ReadFileProgress};
use servo_config::prefs::{PrefValue, PREFS};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[test]
fn test_filemanager() {
    let filemanager = FileManager::new(create_embedder_proxy());
    PREFS.set("dom.testing.htmlinputelement.select_files.enabled", PrefValue::Boolean(true));

    // Try to open a dummy file "components/net/tests/test.jpeg" in tree
    let mut handler = File::open("tests/test.jpeg").expect("test.jpeg is stolen");
    let mut test_file_content = vec![];

    handler.read_to_end(&mut test_file_content)
           .expect("Read components/net/tests/test.jpeg error");

    let patterns = vec![FilterPattern(".txt".to_string())];
    let origin = "test.com".to_string();

    {
        // Try to select a dummy file "components/net/tests/test.jpeg"
        let (tx, rx) = ipc::channel().unwrap();
        filemanager.handle(FileManagerThreadMsg::SelectFile(patterns.clone(), tx, origin.clone(),
            Some("tests/test.jpeg".to_string())));
        let selected = rx.recv().expect("Broken channel")
                                .expect("The file manager failed to find test.jpeg");

        // Expecting attributes conforming the spec
        assert_eq!(selected.filename, PathBuf::from("test.jpeg"));
        assert_eq!(selected.type_string, "image/jpeg".to_string());

        // Test by reading, expecting same content
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            filemanager.handle(FileManagerThreadMsg::ReadFile(tx2, selected.id.clone(), false, origin.clone()));

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
            filemanager.handle(FileManagerThreadMsg::DecRef(selected.id.clone(), origin.clone(), tx2));

            let ret = rx2.recv().expect("Broken channel");
            assert!(ret.is_ok(), "DecRef is not okay");
        }

        // Test by reading again, expecting read error because we invalidated the id
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            filemanager.handle(FileManagerThreadMsg::ReadFile(tx2, selected.id.clone(), false, origin.clone()));

            let msg = rx2.recv().expect("Broken channel");

            match msg {
                Err(FileManagerThreadError::BlobURLStoreError(BlobURLStoreError::InvalidFileID)) => {},
                other => {
                    assert!(false, "Get unexpected response after deleting the id: {:?}", other);
                }
            }
        }
    }
}
