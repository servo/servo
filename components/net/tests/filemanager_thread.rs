/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use base::Epoch;
use base::id::{TEST_PIPELINE_ID, TEST_WEBVIEW_ID};
use embedder_traits::{
    EmbedderControlId, EmbedderControlResponse, FilePickerRequest, FilterPattern,
};
use ipc_channel::ipc;
use net::async_runtime::init_async_runtime;
use net::embedder::NetEmbedderMsg;
use net::filemanager_thread::FileManager;
use net_traits::blob_url_store::BlobURLStoreError;
use net_traits::filemanager_thread::{
    FileManagerThreadError, FileManagerThreadMsg, ReadFileProgress,
};
use servo_config::prefs::Preferences;
use servo_url::ServoUrl;

use crate::create_embedder_proxy2_and_receiver;

#[test]
fn test_filemanager() {
    let _runtime = init_async_runtime();
    let mut preferences = Preferences::default();
    preferences.dom_testing_html_input_element_select_files_enabled = true;
    servo_config::prefs::set(preferences);

    let (embedder_proxy, embedder_receiver) = create_embedder_proxy2_and_receiver();
    let filemanager = FileManager::new(embedder_proxy);

    // Try to open a dummy file "components/net/tests/test.jpeg" in tree
    let mut handler = File::open("tests/test.jpeg").expect("test.jpeg is stolen");
    let mut test_file_content = vec![];

    handler
        .read_to_end(&mut test_file_content)
        .expect("Read components/net/tests/test.jpeg error");

    let origin = ServoUrl::parse("http://test.com").unwrap().origin();

    {
        // Try to select a dummy file "components/net/tests/test.jpeg"
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let control_id = EmbedderControlId {
            webview_id: TEST_WEBVIEW_ID,
            pipeline_id: TEST_PIPELINE_ID,
            index: Epoch(0),
        };
        let file_picker_request = FilePickerRequest {
            origin: origin.clone(),
            current_paths: vec!["tests/test.jpeg".into()],
            filter_patterns: vec![FilterPattern(".txt".to_string())],
            allow_select_multiple: false,
            accept_current_paths_for_testing: true,
        };
        filemanager.handle(FileManagerThreadMsg::SelectFiles(
            control_id,
            file_picker_request,
            result_sender,
        ));

        loop {
            let message = embedder_receiver
                .recv()
                .expect("Should always read message properly");
            match message {
                NetEmbedderMsg::SelectFiles(_, file_picker_request, response_sender) => {
                    let _ = response_sender.send(Some(file_picker_request.current_paths));
                    break;
                },
                _ => {},
            }
        }

        let selected_files = match result_receiver.recv().expect("Broken channel") {
            EmbedderControlResponse::FilePicker(selected_files) => selected_files,
            _ => unreachable!("Received unexpected EmbedderControlResponse"),
        }
        .expect("Expected to get a list of files from embedder.");

        let selected = selected_files
            .first()
            .expect("Should receive at least one file");

        // Expecting attributes conforming the spec
        assert_eq!(selected.filename, PathBuf::from("test.jpeg"));
        assert_eq!(selected.type_string, "image/jpeg".to_string());

        // Test by reading, expecting same content
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            filemanager.handle(FileManagerThreadMsg::ReadFile(
                tx2,
                selected.id.clone(),
                origin.clone(),
            ));

            let msg = rx2.recv().expect("Broken channel");

            if let ReadFileProgress::Meta(blob_buf) =
                msg.expect("File manager reading failure is unexpected")
            {
                let mut bytes = blob_buf.bytes;

                loop {
                    match rx2
                        .recv()
                        .expect("Broken channel")
                        .expect("File manager reading failure is unexpected")
                    {
                        ReadFileProgress::Meta(_) => {
                            panic!("Invalid FileManager reply");
                        },
                        ReadFileProgress::Partial(mut bytes_in) => {
                            bytes.append(&mut bytes_in);
                        },
                        ReadFileProgress::EOF => {
                            break;
                        },
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
            filemanager.handle(FileManagerThreadMsg::DecRef(
                selected.id.clone(),
                origin.clone(),
                tx2,
            ));

            let ret = rx2.recv().expect("Broken channel");
            assert!(ret.is_ok(), "DecRef is not okay");
        }

        // Test by reading again, expecting read error because we invalidated the id
        {
            let (tx2, rx2) = ipc::channel().unwrap();
            filemanager.handle(FileManagerThreadMsg::ReadFile(
                tx2,
                selected.id.clone(),
                origin.clone(),
            ));

            let msg = rx2.recv().expect("Broken channel");

            match msg {
                Err(FileManagerThreadError::BlobURLStoreError(
                    BlobURLStoreError::InvalidFileID,
                )) => {},
                other => {
                    assert!(
                        false,
                        "Get unexpected response after deleting the id: {:?}",
                        other
                    );
                },
            }
        }
    }
}
