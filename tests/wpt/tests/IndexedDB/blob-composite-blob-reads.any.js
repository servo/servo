// META: title=IDB-backed composite blobs maintain coherency
// META: script=resources/support-promises.js
// META: timeout=long

// This test file is intended to help validate browser handling of complex blob
// scenarios where one or more levels of multipart blobs are used and varying
// IPC serialization strategies may be used depending on various complexity
// heuristics.
//
// A variety of approaches of reading the blob's contents are attempted for
// completeness:
// - `fetch-blob-url`: fetch of a URL created via URL.createObjectURL
//   - Note that this is likely to involve multi-process behavior in a way that
//     the next 2 currently will not unless their Blobs are round-tripped
//     through a MessagePort.
// - `file-reader`: FileReader
// - `direct`: Blob.prototype.arrayBuffer()

function composite_blob_test({ blobCount, blobSize, name }) {
  // NOTE: In order to reduce the runtime of this test and due to the similarity
  // of the "file-reader" mechanism to the "direct", "file-reader" is commented
  // out, but if you are investigating failures detected by this test, you may
  // want to uncomment it.
  for (const mode of ["fetch-blob-url", /*"file-reader",*/ "direct"]) {
    promise_test(async testCase => {
      const key = "the-blobs";
      let memBlobs = [];
      for (let iBlob = 0; iBlob < blobCount; iBlob++) {
        memBlobs.push(new Blob([make_arraybuffer_contents(iBlob, blobSize)]));
      }

      const db = await createDatabase(testCase, db => {
        db.createObjectStore("blobs");
      });

      const write_tx = db.transaction("blobs", "readwrite");
      let store = write_tx.objectStore("blobs");
      store.put(memBlobs, key);
      // Make the blobs eligible for GC which is most realistic and most likely
      // to cause problems.
      memBlobs = null;

      await promiseForTransaction(testCase, write_tx);

      const read_tx = db.transaction("blobs", "readonly");
      store = read_tx.objectStore("blobs");
      const read_req = store.get(key);

      await promiseForTransaction(testCase, read_tx);

      const diskBlobs = read_req.result;
      const compositeBlob = new Blob(diskBlobs);

      if (mode === "fetch-blob-url") {
        const blobUrl = URL.createObjectURL(compositeBlob);
        let urlResp = await fetch(blobUrl);
        let urlFetchArrayBuffer = await urlResp.arrayBuffer();
        urlResp = null;

        URL.revokeObjectURL(blobUrl);
        validate_arraybuffer_contents("fetched URL", urlFetchArrayBuffer, blobCount, blobSize);
        urlFetchArrayBuffer = null;

      } else if (mode === "file-reader") {
        let reader = new FileReader();
        let readerPromise = new Promise(resolve => {
          reader.onload = () => {
            resolve(reader.result);
          }
        })
        reader.readAsArrayBuffer(compositeBlob);

        let readArrayBuffer = await readerPromise;
        readerPromise = null;
        reader = null;

        validate_arraybuffer_contents("FileReader", readArrayBuffer, blobCount, blobSize);
        readArrayBuffer = null;
      } else if (mode === "direct") {
        let directArrayBuffer = await compositeBlob.arrayBuffer();
        validate_arraybuffer_contents("arrayBuffer", directArrayBuffer, blobCount, blobSize);
      }
    }, `Composite Blob Handling: ${name}: ${mode}`);
  }
}

// Create an ArrayBuffer whose even bytes are the index identifier and whose
// odd bytes are a sequence incremented by 3 (wrapping at 256) so that
// discontinuities at power-of-2 boundaries are more detectable.
function make_arraybuffer_contents(index, size) {
  const arr = new Uint8Array(size);
  for (let i = 0, counter = 0; i < size; i += 2, counter = (counter + 3) % 256) {
    arr[i] = index;
    arr[i + 1] = counter;
  }
  return arr.buffer;
}

function validate_arraybuffer_contents(source, buffer, blobCount, blobSize) {
  // Accumulate a list of problems we perceive so we can report what seems to
  // have happened all at once.
  const problems = [];

  const arr = new Uint8Array(buffer);

  const expectedLength = blobCount * blobSize;
  const actualCount = arr.length / blobSize;
  if (arr.length !== expectedLength) {
    problems.push(`ArrayBuffer only holds ${actualCount} blobs' worth instead of ${blobCount}.`);
    problems.push(`Actual ArrayBuffer is ${arr.length} bytes but expected ${expectedLength}`);
  }

  const counterBlobStep = (blobSize / 2 * 3) % 256;
  let expectedBlob = 0;
  let blobSeenSoFar = 0;
  let expectedCounter = 0;
  let counterDrift = 0;
  for (let i = 0; i < arr.length; i += 2) {
    if (arr[i] !== expectedBlob || blobSeenSoFar >= blobSize) {
      if (blobSeenSoFar !== blobSize) {
        problems.push(`Truncated blob ${expectedBlob} after ${blobSeenSoFar} bytes.`);
      } else {
        expectedBlob++;
      }
      if (expectedBlob !== arr[i]) {
        problems.push(`Expected blob ${expectedBlob} but found ${arr[i]}, compensating.`);
        expectedBlob = arr[i];
      }
      blobSeenSoFar = 0;
      expectedCounter = (expectedBlob * counterBlobStep) % 256;
      counterDrift = 0;
    }

    if (arr[i + 1] !== (expectedCounter + counterDrift) % 256) {
      const newDrift = expectedCounter - arr[i + 1];
      problems.push(`In blob ${expectedBlob} at ${blobSeenSoFar + 1} bytes in, counter drift now ${newDrift} was ${counterDrift}`);
      counterDrift = newDrift;
    }

    blobSeenSoFar += 2;
    expectedCounter = (expectedCounter + 3) % 256;
  }

  if (problems.length) {
    assert_true(false, `${source} blob payload problem: ${problems.join("\n")}`);
  } else {
    assert_true(true, `${source} blob payloads validated.`);
  }
}

composite_blob_test({
  blobCount: 16,
  blobSize: 256 * 1024,
  name: "Many blobs",
});
