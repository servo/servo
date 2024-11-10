let log = [];

function expect_log(test, expected_log) {
  test.step_func_done(() => {
    const actual_log = log;
    log = [];
    assert_array_equals(actual_log, expected_log, 'fallback log');
  })();
}

// Results of resolving a specifier using import maps.
const Result = {
  // A failure considered as a fetch error in a module script tree.
  // <script>'s error event is fired.
  FETCH_ERROR: "fetch_error",

  // A failure considered as a parse error in a module script tree.
  // Window's error event is fired.
  PARSE_ERROR: "parse_error",

  // The specifier is considered as a relative or absolute URL.
  // Specifier                 Expected log
  // ------------------------- ----------------------
  // ...?name=foo              log:foo
  // data:...log('foo')        foo
  // Others, e.g. bare/bare    relative:bare/bare
  // ------------------------- ----------------------
  // (The last case assumes a file `bare/bare` that logs `relative:bare/bare`
  // exists)
  URL: "URL",
};

const Handler = {
  // Handlers for <script> element cases.
  // Note that on a parse error both WindowErrorEvent and ScriptLoadEvent are
  // called.
  ScriptLoadEvent: "<script> element's load event handler",
  ScriptErrorEvent: "<script> element's error event handler",
  WindowErrorEvent: "window's error event handler",

  // Handlers for dynamic imports.
  DynamicImportResolve: "dynamic import resolve",
  DynamicImportReject: "dynamic import reject",
};

// Returns a map with Handler.* as the keys.
function getHandlers(t, specifier, expected) {
  let handlers = {};
  handlers[Handler.ScriptLoadEvent] = t.unreached_func("Shouldn't load");
  handlers[Handler.ScriptErrorEvent] =
      t.unreached_func("script's error event shouldn't be fired");
  handlers[Handler.WindowErrorEvent] =
      t.unreached_func("window's error event shouldn't be fired");
  handlers[Handler.DynamicImportResolve] =
    t.unreached_func("dynamic import promise shouldn't be resolved");
  handlers[Handler.DynamicImportReject] =
    t.unreached_func("dynamic import promise shouldn't be rejected");

  if (expected === Result.FETCH_ERROR) {
    handlers[Handler.ScriptErrorEvent] = () => expect_log(t, []);
    handlers[Handler.DynamicImportReject] = () => expect_log(t, []);
  } else if (expected === Result.PARSE_ERROR) {
    let error_occurred = false;
    handlers[Handler.WindowErrorEvent] = () => { error_occurred = true; };
    handlers[Handler.ScriptLoadEvent] = t.step_func(() => {
      // Even if a parse error occurs, load event is fired (after
      // window.onerror is called), so trigger the load handler only if
      // there was no previous window.onerror call.
      assert_true(error_occurred, "window.onerror should be fired");
      expect_log(t, []);
    });
    handlers[Handler.DynamicImportReject] = t.step_func(() => {
      assert_false(error_occurred,
        "window.onerror shouldn't be fired for dynamic imports");
      expect_log(t, []);
    });
  } else {
    let expected_log;
    if (expected === Result.URL) {
      const match_data_url = specifier.match(/data:.*log\.push\('(.*)'\)/);
      const match_log_js = specifier.match(/name=(.*)/);
      if (match_data_url) {
        expected_log = [match_data_url[1]];
      } else if (match_log_js) {
        expected_log = ["log:" + match_log_js[1]];
      } else {
        expected_log = ["relative:" + specifier];
      }
    } else {
      expected_log = [expected];
    }
    handlers[Handler.ScriptLoadEvent] = () => expect_log(t, expected_log);
    handlers[Handler.DynamicImportResolve] = () => expect_log(t, expected_log);
  }
  return handlers;
}

// Creates an <iframe> and run a test inside the <iframe>
// to separate the module maps and import maps in each test.
function testInIframe(importMapString, importMapBaseURL, testScript) {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  if (!importMapBaseURL) {
    importMapBaseURL = document.baseURI;
  }
  let content = `
    <script src="/resources/testharness.js"></script>
    <script src="/import-maps/resources/test-helper.js"></script>
    <base href="${importMapBaseURL}">
    <script type="importmap">${importMapString}</script>
    <body>
    <script>
    setup({ allow_uncaught_exception: true });
    ${testScript}
    </sc` + `ript>
  `;
  iframe.contentDocument.write(content);
  iframe.contentDocument.close();
  return fetch_tests_from_window(iframe.contentWindow);
}

function testScriptElement(importMapString, importMapBaseURL, specifier, expected, type) {
  return testInIframe(importMapString, importMapBaseURL, `
    const t = async_test("${specifier}: <script src type=${type}>");
    const handlers = getHandlers(t, "${specifier}", "${expected}");
    const script = document.createElement("script");
    script.setAttribute("type", "${type}");
    script.setAttribute("src", "${specifier}");
    script.addEventListener("load", handlers[Handler.ScriptLoadEvent]);
    script.addEventListener("error", handlers[Handler.ScriptErrorEvent]);
    window.addEventListener("error", handlers[Handler.WindowErrorEvent]);
    document.body.appendChild(script);
  `);
}

function testStaticImport(importMapString, importMapBaseURL, specifier, expected) {
  return testInIframe(importMapString, importMapBaseURL, `
    const t = async_test("${specifier}: static import");
    const handlers = getHandlers(t, "${specifier}", "${expected}");
    const script = document.createElement("script");
    script.setAttribute("type", "module");
    script.setAttribute("src",
        "/import-maps/static-import.py?url=" +
        encodeURIComponent("${specifier}"));
    script.addEventListener("load", handlers[Handler.ScriptLoadEvent]);
    script.addEventListener("error", handlers[Handler.ScriptErrorEvent]);
    window.addEventListener("error", handlers[Handler.WindowErrorEvent]);
    document.body.appendChild(script);
  `);
}

function testDynamicImport(importMapString, importMapBaseURL, specifier, expected, type) {
  return testInIframe(importMapString, importMapBaseURL, `
    const t = async_test("${specifier}: dynamic import (from ${type})");
    const handlers = getHandlers(t, "${specifier}", "${expected}");
    const script = document.createElement("script");
    script.setAttribute("type", "${type}");
    script.innerText =
        "import(\\"${specifier}\\")" +
        ".then(handlers[Handler.DynamicImportResolve], " +
        "handlers[Handler.DynamicImportReject]);";
    script.addEventListener("error",
        t.unreached_func("top-level inline script shouldn't error"));
    document.body.appendChild(script);
  `);
}

function testInIframeInjectBase(importMapString, importMapBaseURL, testScript) {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  let content = `
    <script src="/resources/testharness.js"></script>
    <script src="/import-maps/resources/test-helper.js"></script>
    <script src="/import-maps/resources/inject-base.js?pipe=sub&baseurl=${importMapBaseURL}"></script>
    <script type="importmap">
      ${importMapString}
    </script>
    <body>
    <script>
    setup({ allow_uncaught_exception: true });
    ${testScript}
    </sc` + `ript>
  `;
  iframe.contentDocument.write(content);
  iframe.contentDocument.close();
  return fetch_tests_from_window(iframe.contentWindow);
}

function testStaticImportInjectBase(importMapString, importMapBaseURL, specifier, expected) {
  return testInIframeInjectBase(importMapString, importMapBaseURL, `
    const t = async_test("${specifier}: static import with inject <base>");
    const handlers = getHandlers(t, "${specifier}", "${expected}");
    const script = document.createElement("script");
    script.setAttribute("type", "module");
    script.setAttribute("src",
        "/import-maps/static-import.py?url=" +
        encodeURIComponent("${specifier}"));
    script.addEventListener("load", handlers[Handler.ScriptLoadEvent]);
    script.addEventListener("error", handlers[Handler.ScriptErrorEvent]);
    window.addEventListener("error", handlers[Handler.WindowErrorEvent]);
    document.body.appendChild(script);
  `);
}

function testDynamicImportInjectBase(importMapString, importMapBaseURL, specifier, expected, type) {
  return testInIframeInjectBase(importMapString, importMapBaseURL, `
    const t = async_test("${specifier}: dynamic import (from ${type}) with inject <base>");
    const handlers = getHandlers(t, "${specifier}", "${expected}");
    const script = document.createElement("script");
    script.setAttribute("type", "${type}");
    script.innerText =
        "import(\\"${specifier}\\")" +
        ".then(handlers[Handler.DynamicImportResolve], " +
        "handlers[Handler.DynamicImportReject]);";
    script.addEventListener("error",
        t.unreached_func("top-level inline script shouldn't error"));
    document.body.appendChild(script);
  `);
}

function doTests(importMapString, importMapBaseURL, tests) {
  promise_setup(function () {
    return new Promise((resolve) => {
      window.addEventListener("load", async () => {
        for (const specifier in tests) {
          // <script src> (module scripts)
          await testScriptElement(importMapString, importMapBaseURL, specifier, tests[specifier][0], "module");

          // <script src> (classic scripts)
          await testScriptElement(importMapString, importMapBaseURL, specifier, tests[specifier][1], "text/javascript");

          // static imports.
          await testStaticImport(importMapString, importMapBaseURL, specifier, tests[specifier][2]);

          // dynamic imports from a module script.
          await testDynamicImport(importMapString, importMapBaseURL, specifier, tests[specifier][3], "module");

          // dynamic imports from a classic script.
          await testDynamicImport(importMapString, importMapBaseURL, specifier, tests[specifier][3], "text/javascript");
        }
        done();
        resolve();
      });
    });
  }, { explicit_done: true });
}

function test_loaded(specifier, expected_log, description) {
  promise_test(async t => {
    log = [];
    await import(specifier);
    assert_array_equals(log, expected_log);
  }, description);
};
