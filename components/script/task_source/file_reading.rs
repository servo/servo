/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::domexception::DOMErrorName;
use dom::filereader::{FileReader, TrustedFileReader, GenerationId, ReadMetaData};
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory, ScriptChan};
use std::sync::Arc;
use task::{TaskCanceller, TaskOnce};
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct FileReadingTaskSource(pub Box<ScriptChan + Send + 'static>);

impl Clone for FileReadingTaskSource {
    fn clone(&self) -> FileReadingTaskSource {
        FileReadingTaskSource(self.0.clone())
    }
}

impl TaskSource for FileReadingTaskSource {
    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        self.0.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::FileRead,
            Box::new(canceller.wrap_task(task)),
        ))
    }
}

impl TaskOnce for FileReadingTask {
    fn run_once(self) {
        self.handle_task();
    }
}

#[allow(dead_code)]
pub enum FileReadingTask {
    ProcessRead(TrustedFileReader, GenerationId),
    ProcessReadData(TrustedFileReader, GenerationId),
    ProcessReadError(TrustedFileReader, GenerationId, DOMErrorName),
    ProcessReadEOF(TrustedFileReader, GenerationId, ReadMetaData, Arc<Vec<u8>>),
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
