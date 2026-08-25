// Compare two Uint8Arrays.
function compareArrays(actual, expected) {
  assert_true(actual instanceof Uint8Array, 'actual is Uint8Array');
  assert_true(expected instanceof Uint8Array, 'expected is Uint8Array');
  assert_equals(actual.byteLength, expected.byteLength, 'lengths equal');
  for (let i = 0; i < expected.byteLength; ++i)
    assert_equals(actual[i], expected[i], `Mismatch at position ${i}.`);
}

// Reads from |reader| until at least |targetLength| is read or the stream is
// closed. The data is returned as a combined Uint8Array.
async function readWithLength(reader, targetLength) {
  const chunks = [];
  let actualLength = 0;

  while (true) {
    let {value, done} = await reader.read();
    chunks.push(value);
    actualLength += value.byteLength;

    if (actualLength >= targetLength || done) {
      // It would be better to allocate |buffer| up front with the number of
      // of bytes expected but this is the best that can be done without a
      // BYOB reader to control the amount of data read.
      const buffer = new Uint8Array(actualLength);
      chunks.reduce((offset, chunk) => {
        buffer.set(chunk, offset);
        return offset + chunk.byteLength;
      }, 0);
      return buffer;
    }
  }
}
