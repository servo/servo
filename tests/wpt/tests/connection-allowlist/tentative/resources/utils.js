const STORE_URL =
    '/connection-allowlist/tentative/resources/key-value-store.py';
// URL with this query is exempted from connection allowlist.
const IGNORE_ALLOWLIST = 'ignore-allowlist=true';

// Reads the value specified by `key` from the key-value store on the server.
async function readValueFromServer(key) {
  const serverUrl = `${STORE_URL}?${IGNORE_ALLOWLIST}&key=${key}`;
  const response = await fetch(serverUrl);
  if (!response.ok)
    throw new Error('An error happened in the server');
  const value = await response.text();

  // The value is not stored in the server.
  if (value === '')
    return {status: false};

  return {status: true, value: value};
}

// Convenience wrapper around the above getter that will wait until a value is
// available on the server.
async function nextValueFromServer(key) {
  // Resolve the key if it is a Promise.
  key = await key;

  while (true) {
    // Fetches the test result from the server.
    const {status, value} = await readValueFromServer(key);
    if (!status) {
      // The test result has not been stored yet. Retry after a while.
      await new Promise(resolve => step_timeout(resolve, 100));
      continue;
    }

    return value;
  }
}
