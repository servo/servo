// Names disallowed by NativeIO
const kBadNativeIoNames = [
  'Uppercase',
  'has-dash',
  'has.dot',
  'has/slash',
  'x'.repeat(101),
  '',
];

const kDefaultCapacity = 1024 * 1024;

// Returns a handle to a newly created file that holds some data.
//
// The file will be closed and deleted when the test ends.
async function createFile(testCase, fileName, data = [64, 65, 66, 67]) {
  const file = await storageFoundation.open(fileName);

  testCase.add_cleanup(async () => {
    await file.close();
    await storageFoundation.delete(fileName);
  });

  const buffer = Uint8Array.from(data);

  const {writtenBytes} = await file.write(buffer, 0);
  assert_equals(writtenBytes, data.length,
    'NativeIOFile.write() should resolve with the number of bytes written');

  return file;
}

// Returns a handle to a newly created file that holds some data.
//
// The file will be closed and deleted when the test ends.
function createFileSync(testCase, fileName, data = [64, 65, 66, 67]) {
  const file = storageFoundation.openSync(fileName);
  testCase.add_cleanup(() => {
    file.close();
    storageFoundation.deleteSync(fileName);
  });

  const buffer = Uint8Array.from(data);
  const {writtenBytes} = file.write(buffer, 0);
  assert_equals(writtenBytes, data.length,
    'NativeIOFileSync.write() should resolve with the number of bytes written');

  return file;
}

// Returns an Uint8Array with pseudorandom data.
//
// The PRNG should be sufficient to defeat compression schemes, but it is not
// cryptographically strong.
function createLargeArray(size, seed) {
  const buffer = new Uint8Array(size);

  // 32-bit xorshift - the seed can't be zero
  let state = 1000 + seed;

  for (let i = 0; i < size; ++i) {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    buffer[i] = state & 0xff;
  }

  return buffer;
}

// Attempts to read the entire file into a buffer.
async function readIoFile(file) {
  const length = await file.getLength();
  const {buffer, readBytes} = await file.read(new Uint8Array(length), 0);
  return buffer;
}

// Attempts to read the entire file into a buffer.
function readIoFileSync(file) {
  const length = file.getLength();
  const {buffer, readBytes} = file.read(new Uint8Array(length), 0);
  return buffer;
}

// Default capacity allocation for non-quota related tests.
async function reserveAndCleanupCapacity(testCase,
                                         capacity = kDefaultCapacity) {
  const grantedCapacity = await storageFoundation.requestCapacity(capacity);
  testCase.add_cleanup(async () => {
    let available_capacity = await storageFoundation.getRemainingCapacity();
    await storageFoundation.releaseCapacity(available_capacity);
  });
  assert_greater_than_equal(grantedCapacity, capacity);
}

// Default capacity allocation for non-quota related sync tests.
function reserveAndCleanupCapacitySync(testCase, capacity = kDefaultCapacity) {
  const grantedCapacity = storageFoundation.requestCapacitySync(capacity);
  testCase.add_cleanup(() => {
    let available_capacity = storageFoundation.getRemainingCapacitySync();
    storageFoundation.releaseCapacitySync(available_capacity);
  });
  assert_greater_than_equal(grantedCapacity, capacity);
}
