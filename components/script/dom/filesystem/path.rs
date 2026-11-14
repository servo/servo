/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Path algrithms from the File and Directory Entries API.
//!
//! <https://wicg.github.io/entries-api/#algorithms>

use crate::dom::bindings::root::DomRoot;
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;
use crate::dom::filesystementry::FileSystemEntry;

/// <https://wicg.github.io/entries-api/#names-paths>
/// A valid path is a sequence of '/' separated path segments,
/// which does not contain contain NUL (U+0000) or '\' (U+005C REVERSE SOLIDUS).
pub(crate) fn is_valid_path(path: &str) -> bool {
    !path.contains('\0') && !path.contains('\\')
}

/// <https://wicg.github.io/entries-api/#resolve-a-relative-path>
pub(crate) fn resolve_relative_path(abspath: &str, path: &str) -> String {
    // Step 1. If path is an absolute path, return path.
    if path.starts_with('/') {
        return path.to_owned();
    }

    // Step 2. Let abspath segments be the result of strictly splitting abspath
    // on '/' (U+002F SOLIDUS).
    let mut abspath_segments: Vec<&str> = abspath.split('/').collect();

    // Step 3. Let path segments be the result of strictly splitting path on
    // '/' (U+002F SOLIDUS).
    //
    // Step 4. For each segment in path segments, switch on segment:
    for segment in path.split('/') {
        match segment {
            // empty string: Continue.
            "" => {},
            // '.' (U+002E FULL STOP): Continue.
            "." => {},
            // '..' (U+002E FULL STOP, U+002E FULL STOP): Remove the last member
            // of abspath segments unless it is the only member.
            ".." => {
                if abspath_segments.len() > 1 {
                    abspath_segments.pop();
                }
            },
            // Otherwise: Append segment to abspath segments.
            _ => abspath_segments.push(segment),
        }
    }

    // Step 5. Return abspath segments joined by '/' (U+002F SOLIDUS).
    //
    // We do something different from spec which is broken:
    // <https://github.com/WICG/entries-api/issues/50>
    let mut result = String::new();
    for segment in abspath_segments
        .iter()
        .filter(|segment| !segment.is_empty())
    {
        result.push('/');
        result.push_str(segment);
    }
    if result.is_empty() {
        result.push('/');
    }
    result
}

/// <https://wicg.github.io/entries-api/#evaluate-a-path>
pub(crate) fn evaluate_path(
    root: &FileSystemDirectoryEntry,
    path: &str,
) -> Option<DomRoot<FileSystemEntry>> {
    // Step 1. Let segments be the result of strictly splitting path on '/'
    // (U+002F SOLIDUS).

    // Step 2. Remove the first entry from segments.
    let mut segments = path.split('/').skip(1).peekable();

    // The directory starts at the root; the stack tracks the directories
    // traversed so far so that '..' can walk back up.
    let mut directories = vec![DomRoot::from_ref(root)];

    // Step 3. For each segment in segments, switch on segment:
    while let Some(segment) = segments.next() {
        match segment {
            // empty string: Continue.
            "" => {},
            // '.' (U+002E FULL STOP): Continue.
            "." => {},
            // '..' (U+002E FULL STOP, U+002E FULL STOP): Let directory be
            // directory's parent, or directory if none.
            ".." => {
                if directories.len() > 1 {
                    directories.pop();
                }
            },
            // Otherwise: Run these steps:
            name => {
                let directory = directories.last().expect("directories is never empty");
                // 1. Let item be the member of directory with name equal to
                // segment, or return failure if none.
                let item = directory.child(name)?;
                // 2. If segment is the last item in segments, return item.
                if segments.peek().is_none() {
                    return Some(item);
                }
                // 3. If item is a file, return failure.
                let directory = DomRoot::downcast::<FileSystemDirectoryEntry>(item)?;
                // 4. Let directory be item.
                directories.push(directory);
            },
        }
    }

    // A path ending in '.' or '..' never returns from inside the loop; the
    // directory reached so far is the result.
    let directory = directories.pop().expect("directories is never empty");
    Some(DomRoot::upcast::<FileSystemEntry>(directory))
}
