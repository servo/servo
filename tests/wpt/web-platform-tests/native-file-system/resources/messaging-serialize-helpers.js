'use strict';

// This script depends on the following script:
//    /native-file-system/resources/test-helpers.js

// Serializes an array of FileSystemHandles where each element can be either a
// FileSystemFileHandle or FileSystemDirectoryHandle.
async function serialize_handles(handle_array) {
  const serialized_handle_array = [];
  for (let i = 0; i < handle_array.length; ++i) {
    serialized_handle_array.push(await serialize_handle(handle_array[i]));
  }
  return serialized_handle_array;
}

// Serializes either a FileSystemFileHandle or FileSystemDirectoryHandle.
async function serialize_handle(handle) {
  let serialized;
  if (handle.isDirectory) {
    serialized = await serialize_file_system_directory_handle(handle);
  } else if (handle.isFile) {
    serialized = await serialize_file_system_file_handle(handle);
  } else {
    throw 'Object is not a FileSystemFileHandle or ' +
    `FileSystemDirectoryHandle ${handle}`;
  }
  return serialized;
}

// Creates a dictionary for a FileSystemHandle base, which contains
// serialized properties shared by both FileSystemFileHandle and
// FileSystemDirectoryHandle.
async function serialize_file_system_handle(handle) {
  const read_permission =
    await handle.queryPermission({ writable: false });

  const write_permission =
    await handle.queryPermission({ writable: true })

  return {
    is_file: handle.isFile,
    is_directory: handle.isDirectory,
    name: handle.name,
    read_permission,
    write_permission
  };
}

// Create a dictionary with each property value in FileSystemFileHandle.
// Also, reads the contents of the file to include with the returned
// dictionary.  Example output:
// {
//   is_file: true,
//   is_directory: false,
//   name: "example-file-name"
//   read_permission: "granted",
//   write_permission: "granted",
//   contents: "example-file-contents"
// }
async function serialize_file_system_file_handle(file_handle) {
  const contents = await getFileContents(file_handle);

  const serialized_file_system_handle =
    await serialize_file_system_handle(file_handle);

  return Object.assign(serialized_file_system_handle, { contents });
}

// Create a dictionary with each property value in FileSystemDirectoryHandle.
// Example output:
// {
//   is_file: false,
//   is_directory: true,
//   name: "example-directory-name"
//   read_permission: "granted",
//   write_permission: "granted",
//   files: [<first serialized file>, ...]
//   directories: [<first serialized subdirectory>, ...]
// }
async function serialize_file_system_directory_handle(directory_handle) {
  // Serialize the contents of the directory.
  const serialized_files = [];
  const serialized_directories = [];
  for await (const child_handle of directory_handle.getEntries()) {
    const serialized_child_handle = await serialize_handle(child_handle);
    if (child_handle.isDirectory) {
      serialized_directories.push(serialized_child_handle);
    } else {
      serialized_files.push(serialized_child_handle);
    }
  }

  // Order the serialized contents of the directory by name.
  serialized_files.sort((left, right) => {
    return left.name.localeCompare(right.name);
  });
  serialized_directories.sort((left, right) => {
    return left.name.localeCompare(right.name);
  });

  // Serialize the directory's common properties shared by all
  // FileSystemHandles.
  const serialized_file_system_handle =
    await serialize_file_system_handle(directory_handle);

  return Object.assign(
    serialized_file_system_handle,
    { files: serialized_files, directories: serialized_directories });
}

// Verifies |left_array| is a clone of |right_array| where each element
// is a cloned FileSystemHandle with the same properties and contents.
async function assert_equals_cloned_handles(left_array, right_array) {
  assert_equals(left_array.length, right_array.length,
    'Each array of FileSystemHandles must have the same length');

  for (let i = 0; i < left_array.length; ++i) {
    assert_not_equals(left_array[i], right_array[i],
      'Clones must create new FileSystemHandle instances.');

    const left_serialized = await serialize_handle(left_array[i]);
    const right_serialized = await serialize_handle(right_array[i]);
    assert_equals_serialized_handle(left_serialized, right_serialized);
  }
}

// Verifies |left_array| is the same as |right_array| where each element
// is a serialized FileSystemHandle with the same properties.
function assert_equals_serialized_handles(left_array, right_array) {
  assert_equals(left_array.length, right_array.length,
    'Each array of serialized handles must have the same length');

  for (let i = 0; i < left_array.length; ++i) {
    assert_equals_serialized_handle(left_array[i], right_array[i]);
  }
}

// Verifies each property of a serialized FileSystemFileHandle or
// FileSystemDirectoryHandle.
function assert_equals_serialized_handle(left, right) {
  if (left.is_directory) {
    assert_equals_serialized_file_system_directory_handle(left, right);
  } else if (left.is_file) {
    assert_equals_serialized_file_system_file_handle(left, right);
  } else {
    throw 'Object is not a FileSystemFileHandle or ' +
    `FileSystemDirectoryHandle ${left}`;
  }
}

// Compares the output of serialize_file_system_handle() for
// two FileSystemHandles.
function assert_equals_serialized_file_system_handle(left, right) {
  assert_equals(left.is_file, right.is_file,
    'Each FileSystemHandle instance must use the expected "isFile".');

  assert_equals(left.is_directory, right.is_directory,
    'Each FileSystemHandle instance must use the expected "isDirectory".');

  assert_equals(left.name, right.name,
    'Each FileSystemHandle instance must use the expected "name" ' +
    ' property.');

  assert_equals(left.read_permission, right.read_permission,
    'Each FileSystemHandle instance must have the expected read ' +
    ' permission.');

  assert_equals(left.write_permission, right.write_permission,
    'Each FileSystemHandle instance must have the expected write ' +
    ' permission.');
}

// Compares the output of serialize_file_system_file_handle()
// for two FileSystemFileHandle.
function assert_equals_serialized_file_system_file_handle(left, right) {
  assert_equals_serialized_file_system_handle(left, right);
  assert_equals(left.contents, right.contents,
    'Each FileSystemFileHandle instance must have the same contents.');
}

// Compares the output of serialize_file_system_directory_handle()
// for two FileSystemDirectoryHandles.
function assert_equals_serialized_file_system_directory_handle(left, right) {
  assert_equals_serialized_file_system_handle(left, right);

  assert_equals(left.files.length, right.files.length,
    'Each FileSystemDirectoryHandle must contain the same number of ' +
    'file children');

  for (let i = 0; i < left.files.length; ++i) {
    assert_equals_serialized_file_system_file_handle(
      left.files[i], right.files[i]);
  }

  assert_equals(left.directories.length, right.directories.length,
    'Each FileSystemDirectoryHandle must contain the same number of ' +
    'directory children');

  for (let i = 0; i < left.directories.length; ++i) {
    assert_equals_serialized_file_system_directory_handle(
      left.directories[i], right.directories[i]);
  }
}

// Creates a dictionary with interesting property values from MessageEvent.
function serialize_message_error_event(message_error_event) {
  return {
    data: message_error_event.data,
    origin: message_error_event.origin,
    last_event_id: message_error_event.lastEventId,
    has_source: (message_error_event.source !== null),
    ports_length: message_error_event.ports.length
  };
}

// Compares the output of serialize_message_error_event() with an
// expected result.
function assert_equals_serialized_message_error_event(
  serialized_event, expected_origin, expected_has_source) {
  assert_equals(serialized_event.data, null,
    'The message error event must set the "data" property to null.');

  assert_equals(serialized_event.origin, expected_origin,
    'The message error event must have the expected "origin" property.');

  assert_equals(serialized_event.last_event_id, "",
    'The message error event must set the "lastEventId" property to the empty string.');

  assert_equals(serialized_event.has_source, expected_has_source,
    'The message error event must have the expected "source" property.');

  assert_equals(serialized_event.ports_length, 0,
    'The message error event must not contain any message ports.');
}
