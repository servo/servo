
const kDefaultDictionaryContent = 'This is a test dictionary.\n';
const kDefaultDictionaryHashBase64 =
    ':U5abz16WDg7b8KS93msLPpOB4Vbef1uRzoORYkJw9BY=:';
const kRegisterDictionaryPath = './resources/register-dictionary.py';
const kCompressedDataPath = './resources/compressed-data.py';
const kExpectedCompressedData =
    `This is compressed test data using a test dictionary`;
const kCheckAvailableDictionaryHeaderMaxRetry = 10;
const kCheckAvailableDictionaryHeaderRetryTimeout = 200;
const kCheckPreviousRequestHeadersMaxRetry = 5;
const kCheckPreviousRequestHeadersRetryTimeout = 250;

// Gets the remote URL corresponding to `relative_path`.
function getRemoteHostUrl(relative_path) {
  const remote_origin = new URL(get_host_info().HTTPS_REMOTE_ORIGIN);
  let result = new URL(relative_path, location.href);
  result.protocol = remote_origin.protocol;
  result.hostname = remote_origin.hostname;
  result.port = remote_origin.port;
  return result.href;
}

// Calculates the Structured Field Byte Sequence containing the SHA-256 hash of
// the contents of the dictionary text.
async function calculateDictionaryHash(dictionary_text) {
  const encoded = (new TextEncoder()).encode(dictionary_text);
  const digest = await crypto.subtle.digest('SHA-256', encoded)
  return ':' + btoa(String.fromCharCode(...new Uint8Array(digest))) + ':';
}

// Checks the HTTP request headers which is sent to the server.
async function checkHeaders(check_remote = false) {
  let url = './resources/echo-headers.py';
  if (check_remote) {
    url = getRemoteHostUrl(url);
  }
  return await (await fetch(url)).json();
}

// Checks the "available-dictionary" header in the HTTP request headers.
async function checkAvailableDictionaryHeader(check_remote = false) {
  return (await checkHeaders(check_remote))['available-dictionary'];
}

// Waits until the "available-dictionary" header is available in the HTTP
// request headers, and returns the header. If the header is not available after
// the specified number of retries, returns an error message. If the
// `expected_header` is specified, this method waits until the header is
// available and matches the `expected_header`.
async function waitUntilAvailableDictionaryHeader(test, {
  max_retry = kCheckAvailableDictionaryHeaderMaxRetry,
  expected_header = undefined,
  check_remote = false
}) {
  for (let retry_count = 0; retry_count <= max_retry; retry_count++) {
    const header = await checkAvailableDictionaryHeader(check_remote);
    if (header) {
      if (expected_header === undefined || header == expected_header) {
        return header;
      }
    }
    await new Promise(
        (resolve) => test.step_timeout(
            resolve, kCheckAvailableDictionaryHeaderRetryTimeout));
  }
  return '"available-dictionary" header is not available';
}

// Checks the HTTP request headers which was sent to the server with `token`
// to register a dictionary.
async function checkPreviousRequestHeaders(token, check_remote = false) {
  let url = `./resources/register-dictionary.py?get_previous_header=${token}`;
  if (check_remote) {
    url = getRemoteHostUrl(url);
  }
  return await (await fetch(url)).json();
}

// Waits until the HTTP request headers which was sent to the server with
// `token` to register a dictionary is available, and returns the header. If the
// header is not available after the specified number of retries, returns
// `undefined`.
async function waitUntilPreviousRequestHeaders(
    test, token, check_remote = false) {
  for (let retry_count = 0; retry_count <= kCheckPreviousRequestHeadersMaxRetry;
       retry_count++) {
    const header =
        (await checkPreviousRequestHeaders(token, check_remote))['headers'];
    if (header) {
      return header;
    }
    await new Promise(
        (resolve) => test.step_timeout(
            resolve, kCheckPreviousRequestHeadersRetryTimeout));
  }
  return undefined;
}

// Clears the site data for the specified directive by sending a request to
// `./resources/clear-site-data.py` which returns `Clear-Site-Data` response
// header.
// Note: When `directive` is 'cache' or 'cookies' is specified, registered
// compression dictionaries should be also cleared.
async function clearSiteData(directive = 'cache') {
  return await (await fetch(
                    `./resources/clear-site-data.py?directive=${directive}`))
      .text();
}

// A utility test method that adds the `clearSiteData()` method to the
// testharness cleanup function. This is intended to ensure that registered
// dictionaries are cleared in tests and that registered dictionaries do not
// interfere with subsequent tests.
function compression_dictionary_promise_test(func, name, properties) {
  promise_test(async (test) => {
    test.add_cleanup(clearSiteData);
    await func(test);
  }, name, properties);
}
