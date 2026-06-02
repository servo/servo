function header(name, value) {
  return `header(${name},${value})`;
}

function contentType(type) {
  return header("Content-Type", type);
}

function contentTypeOptions(type) {
  return header("X-Content-Type-Options", type);
}

function testFetchNoCors(_t, path, { headers }) {
  return fetch(path, {
    ...(headers ? { headers } : {}),
    mode: "no-cors",
  });
}

function testElementInitiator(t, path, name) {
  let element = document.createElement(name);
  element.src = path;
  t.add_cleanup(() => element.remove());
  return new Promise((resolve, reject) => {
    element.onerror = e => reject(new TypeError());
    element.onload = resolve;

    document.body.appendChild(element);
  });
}

function testImageInitiator(t, path) {
  return testElementInitiator(t, path, "img");
}

function testAudioInitiator(t, path) {
  return testElementInitiator(t, path, "audio");
}

function testVideoInitiator(t, path) {
  return testElementInitiator(t, path, "video");
}

function testScriptInitiator(t, path) {
  return testElementInitiator(t, path, "script");
}

function runTest(t, test, file, options, ...pipe) {
  const path = `${file}${pipe.length ? `?pipe=${pipe.join("|")}` : ""}`;
  return test(t, path, options)
}

function testRunAll(file, testCallback, adapter, options) {
  let testcase = function (test, message, skip) {
    return {test, message, skip};
  };

  const name = "...";
  [ testcase(testFetchNoCors, `fetch(${name}, {mode: "no-cors"})`, false || options.skip.includes("fetch")),
    testcase(testImageInitiator, `<img src=${name}>`,  options.onlyFetch || options.skip.includes("image")),
    testcase(testAudioInitiator, `<audio src=${name}>`, options.onlyFetch || options.skip.includes("audio")),
    testcase(testVideoInitiator, `<video src=${name}>`, options.onlyFetch || options.skip.includes("video")),
    testcase(testScriptInitiator, `<script src=${name}>`, options.onlyFetch || options.skip.includes("script")),
  ].filter(({skip}) => !skip)
  .forEach(({test, message}) => {
    testCallback((t, ...args) => adapter(t, runTest(t, test, file, options, ...args), message), header => `${header}: ${message}`);
  });
}

function expected_block(file, testCallback, options = {}) {
  let defaultOptions = {
    onlyFetch: !self.GLOBAL.isWindow(),
    skip: []
  };
  testRunAll(file, testCallback, (t, promise, message) => promise_rejects_js(t, TypeError, promise, message), { ...defaultOptions, ...options });
}

function expected_allow(file, testCallback, options = {}) {
  let defaultOptions = {
    onlyFetch: !self.GLOBAL.isWindow(),
    skip: [],
    headers: null
  };
  testRunAll(file, testCallback, (_t, promise, _message) => promise, { ...defaultOptions, ...options });
}

function expected_allow_fetch(file, testCallback, options = {}) {
  let defaultOptions = {
    skip: [],
    headers: null,
  };
  testRunAll(file, testCallback, (_t, promise, _message) => promise, { ...defaultOptions, ...options, onlyFetch: true });
}

function expected_block_fetch(file, testCallback, options = {}) {
  let defaultOptions = {
    skip: [],
    headers: null,
  };
  testRunAll(file, testCallback, (t, promise, message) => promise_rejects_js(t, TypeError, promise, message), { ...defaultOptions, ...options, onlyFetch: true });
}
