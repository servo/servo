/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::domexception::DOMErrorName;
use dom::filereader::{FileReader, TrustedFileReader, GenerationId, ReadMetaData};
use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory, ScriptChan};
use std::marker::PhantomData;
use std::sync::Arc;
use task::{TaskCanceller, TaskOnce};
use task_source::{TaskSource, TaskSourceName};
use typeholder::TypeHolderTrait;

#[derive(JSTraceable)]
pub struct FileReadingTaskSource<TH: TypeHolderTrait>(
    pub Box<ScriptChan + Send + 'static>,
    pub PipelineId,
    pub PhantomData<TH>
);

impl<TH: TypeHolderTrait> Clone for FileReadingTaskSource<TH> {
    fn clone(&self) -> FileReadingTaskSource<TH> {
        FileReadingTaskSource(self.0.clone(), self.1.clone(), Default::default())
    }
}

impl<TH: TypeHolderTrait> TaskSource for FileReadingTaskSource<TH> {
    const NAME: TaskSourceName = TaskSourceName::FileReading;

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
            Some(self.1),
        ))
    }
}

impl<TH: TypeHolderTrait> TaskOnce for FileReadingTask<TH> {
    fn run_once(self) {
        self.handle_task();
    }
}

#[allow(dead_code)]
pub enum FileReadingTask<TH: TypeHolderTrait> {
    ProcessRead(TrustedFileReader<TH>, GenerationId),
    ProcessReadData(TrustedFileReader<TH>, GenerationId),
    ProcessReadError(TrustedFileReader<TH>, GenerationId, DOMErrorName),
    ProcessReadEOF(TrustedFileReader<TH>, GenerationId, ReadMetaData, Arc<Vec<u8>>),
}

impl<TH: TypeHolderTrait> FileReadingTask<TH> {
    pub fn handle_task(self) {
        use self::FileReadingTask::*;

        match self {
            ProcessRead(reader, gen_id) =>
                FileReader::<TH>::process_read(reader, gen_id),
            ProcessReadData(reader, gen_id) =>
                FileReader::<TH>::process_read_data(reader, gen_id),
            ProcessReadError(reader, gen_id, error) =>
                FileReader::<TH>::process_read_error(reader, gen_id, error),
            ProcessReadEOF(reader, gen_id, metadata, blob_contents) =>
                FileReader::<TH>::process_read_eof(reader, gen_id, metadata, blob_contents),
        }
    }
}
