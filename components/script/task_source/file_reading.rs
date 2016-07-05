/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::blob::DataSlice;
use dom::domexception::DOMErrorName;
use dom::filereader::{FileReader, TrustedFileReader, GenerationId, ReadMetaData};
use script_thread::MainThreadScriptMsg;
use std::sync::mpsc::Sender;
use task_source::TaskSource;

#[derive(JSTraceable, Clone)]
pub struct FileReadingTaskSource(pub Sender<MainThreadScriptMsg>);

impl TaskSource<FileReadingTask> for FileReadingTaskSource {
    fn queue(&self, msg: FileReadingTask) -> Result<(), ()> {
        self.0.send(MainThreadScriptMsg::FileReading(msg)).map_err(|_| ())
    }
}

pub enum FileReadingTask {
    ProcessRead(TrustedFileReader, GenerationId),
    ProcessReadData(TrustedFileReader, GenerationId),
    ProcessReadError(TrustedFileReader, GenerationId, DOMErrorName),
    ProcessReadEOF(TrustedFileReader, GenerationId, ReadMetaData, DataSlice),
}

impl FileReadingTask {
    pub fn handle_task(self) {
        use self::FileReadingTask::*;

        match self {
            ProcessRead(reader, gen_id) =>
                FileReader::process_read(reader, gen_id),
            ProcessReadData(reader, gen_id) =>
                FileReader::process_read_data(reader, gen_id),
            ProcessReadError(reader, gen_id, error) =>
                FileReader::process_read_error(reader, gen_id, error),
            ProcessReadEOF(reader, gen_id, metadata, blob_contents) =>
                FileReader::process_read_eof(reader, gen_id, metadata, blob_contents),
        }
    }
}
