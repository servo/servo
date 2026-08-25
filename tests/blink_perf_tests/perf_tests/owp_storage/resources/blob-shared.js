let totalBytes = 0;
let errors = [];

function showManualInput() {
  document.body.innerHTML =
  `
  <h4><b>Manual Input:</b></h4>
  <form>
    Blob size: <input type="text" id="blob_size"><br>
    Number of blobs: <input type="text" id="num_blobs"><br>
    <input type="button" value="Start Benchmark" onclick="getParams();" />
  </form>
  <h4><b>Benchmark Output:</b></h4>
  `;
}

function recordError(message) {
  console.log(message);
  errors.push(message);

  let error = document.createElement('div');
  error.textContent = message;
  document.body.appendChild(error);
}

function createBlob(size) {
  let blob = new Blob([new Uint8Array(size)],
                      {type: 'application/octet-string'});
  totalBytes += size;
  return blob;
}

function readBlobAsync(blob) {
  const reader = new FileReader();
  return new Promise(resolve => {
    reader.onerror = recordError;
    reader.onloadend = e => { resolve(reader); };
    reader.readAsArrayBuffer(blob);
  });
}

async function createAndRead(size) {
  let blob = new Blob([new Uint8Array(size)],
                      {type: 'application/octet-string'});
  const reader = await readBlobAsync(blob);
  if (reader.error)
    recordError(`Error reading blob: ${reader.error}`);
  else if (reader.result.byteLength != size)
    recordError('Error reading blob: Blob size does not match.');
}

let readBlobAsArrayBuffer = (blob, callback) => {
  const reader = new FileReader();
  reader.onerror = recordError;
  reader.onloadend = () => {
    if (reader.error) {
      recordError(`Error reading blob: ${reader.error}`);
    } else {
      callback(reader.result);
    }
  };
  reader.readAsArrayBuffer(blob);
}

async function createBlobAndImmediatelyRead(numBlobs, size) {
  let start = performance.now();
  errors = [];

  logToDocumentBody(`Creating and reading ${numBlobs} blobs...`);
  for (let i = 0; i < numBlobs; i++)
    await createAndRead(size);
  logToDocumentBody('Finished.');

  if (errors.length)
    logToDocumentBody('Errors on page: ' + errors.join(', '));
}

async function createBlobsAndReadInParallel(numBlobs, size) {
  errors = [];

  logToDocumentBody(`Creating and reading ${numBlobs} blobs...`);
  await new Promise(resolve => {
    let totalSizeRead = 0;
    let blobsRead = 0;
    let blobReadCallback = array => {
      blobsRead += 1;
      totalSizeRead += array.byteLength;
      if (blobsRead == numBlobs) {
        if (totalSizeRead != numBlobs * size) {
          recordError(`Error reading blob, total sizes don't match ${totalSizeRead} vs ${numBlobs * size}`);
        }
        logToDocumentBody('Done reading all blobs.');
        resolve();
      }
    }

    for (let i = 0; i < numBlobs; i++) {
      let blob = createBlob(size);
      readBlobAsArrayBuffer(blob, blobReadCallback);
    }
  });

  if (errors.length) {
    let errorStr = errors.join(', ');
    logToDocumentBody('Errors on page: ' + errorStr);
    reportError(errorStr);
  }
}
