/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A module for writing time profiler traces out to a self contained HTML file.

use profile_traits::time::{ProfilerCategory, TimerMetadata};
use serde_json;
use std::fs;
use std::io::Write;

/// An RAII class for writing the HTML trace dump.
pub struct TraceDump {
    file: fs::File,
}

#[derive(Debug, Serialize)]
struct TraceEntry {
    category: ProfilerCategory,
    metadata: Option<TimerMetadata>,

    #[serde(rename = "startTime")]
    start_time: u64,

    #[serde(rename = "endTime")]
    end_time: u64,

    #[serde(rename = "startEnergy")]
    start_energy: u64,

    #[serde(rename = "endEnergy")]
    end_energy: u64,
}

impl TraceDump {
    /// Create a new TraceDump and write the prologue of the HTML file out to
    /// disk.
    pub fn new(mut file: fs::File) -> TraceDump {
        write_prologue(&mut file);
        TraceDump { file: file }
    }

    /// Write one trace to the trace dump file.
    pub fn write_one(&mut self,
                     category: &(ProfilerCategory, Option<TimerMetadata>),
                     time: (u64, u64),
                     energy: (u64, u64)) {
        let entry = TraceEntry {
            category: category.0,
            metadata: category.1.clone(),
            start_time: time.0,
            end_time: time.1,
            start_energy: energy.0,
            end_energy: energy.1,
        };
        serde_json::to_writer(&mut self.file, &entry).unwrap();
        writeln!(&mut self.file, ",").unwrap();
    }
}

impl Drop for TraceDump {
    /// Write the epilogue of the trace dump HTML file out to disk on
    /// destruction.
    fn drop(&mut self) {
        write_epilogue(&mut self.file);
    }
}

fn write_prologue(file: &mut fs::File) {
    writeln!(file, "{}", include_str!("./trace-dump-prologue-1.html")).unwrap();
    writeln!(file, "{}", include_str!("./trace-dump.css")).unwrap();
    writeln!(file, "{}", include_str!("./trace-dump-prologue-2.html")).unwrap();
}

fn write_epilogue(file: &mut fs::File) {
    writeln!(file, "{}", include_str!("./trace-dump-epilogue-1.html")).unwrap();
    writeln!(file, "{}", include_str!("./trace-dump.js")).unwrap();
    writeln!(file, "{}", include_str!("./trace-dump-epilogue-2.html")).unwrap();
}
