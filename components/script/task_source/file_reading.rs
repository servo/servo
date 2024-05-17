/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::PipelineId;

use crate::dom::domexception::DOMErrorName;
use crate::dom::filereader::{FileReader, GenerationId, ReadMetaData, TrustedFileReader};
use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct FileReadingTaskSource(
    pub Box<dyn ScriptChan + Send + 'static>,
    #[no_trace] pub PipelineId,
);

impl Clone for FileReadingTaskSource {
    fn clone(&self) -> FileReadingTaskSource {
        FileReadingTaskSource(self.0.clone(), self.1)
    }
}

impl TaskSource for FileReadingTaskSource {
    const NAME: TaskSourceName = TaskSourceName::FileReading;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        self.0.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::FileRead,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            FileReadingTaskSource::NAME,
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
    ProcessReadEOF(TrustedFileReader, GenerationId, ReadMetaData, Vec<u8>),
}

impl FileReadingTask {
    pub fn handle_task(self) {
        use self::FileReadingTask::*;

        match self {
            ProcessRead(reader, gen_id) => FileReader::process_read(reader, gen_id),
            ProcessReadData(reader, gen_id) => FileReader::process_read_data(reader, gen_id),
            ProcessReadError(reader, gen_id, error) => {
                FileReader::process_read_error(reader, gen_id, error)
            },
            ProcessReadEOF(reader, gen_id, metadata, blob_contents) => {
                FileReader::process_read_eof(reader, gen_id, metadata, blob_contents)
            },
        }
    }
}
