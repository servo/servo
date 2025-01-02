// META: script=/common/get-host-info.sub.js
//
// A test to verify that, when a srcdoc frame is created when it's parent
// has a non-default base URI, and is later restored from history, it has
// the same baseURI it started with, even if the parent has changed its own
// baseURI in the meantime.
// The parent always communicates with the child via postMessage since some
// of the test cases include the child being sandboxed.

async function sendMessage(frame, msg) {
  const result = new Promise(r => onmessage = e => r(e.data));
  frame.postMessage(msg, "*");
  return await result;
}

const runTest = (description, sandbox_flags) => {
  promise_test(async test => {
    const original_parent_baseURI = document.baseURI;
    // Create a URL for the child frame to navigate to.
    const nav_target =
        (new URL('./resources/send-back-base-url.html', location.href)).href;

    // Change parent to a non-default baseURI.
    const base_element = document.createElement("base");
    base_element.href = get_host_info().REMOTE_ORIGIN;
    document.head.appendChild(base_element);
    assert_not_equals(document.baseURI, original_parent_baseURI,
        "parent baseURI failed to change.");
    const non_default_parent_baseURI = document.baseURI;

    // Create child and load a srcdoc frame.
    const iframe = document.createElement("iframe");
    if (sandbox_flags !== null)
      iframe.sandbox = sandbox_flags;
    iframe.srcdoc = `
      <head>
      <script>
        addEventListener('message', (event) => {
          if (event.data == 'report baseURI')
            event.source.postMessage(document.baseURI, event.origin);
          if (event.data == 'click link')
            document.getElementById('link').click();
        });
        parent.postMessage('loaded', '*');
      </scr`+`ipt>
      </head>
      <body>
      <a id='link' href='` + nav_target+ `'>link</a>
      </body>
    `;

    const child_loaded = new Promise(r => onmessage = e => r(e.data));
    document.body.appendChild(iframe);
    assert_equals(await child_loaded, "loaded");

    // Verify child's baseURI matches parent.
    const child_base_uri = await sendMessage(frames[0], "report baseURI");
    assert_equals(child_base_uri, non_default_parent_baseURI);

    // Navigate child frame to non-srcdoc.
    const child_loaded2 = await sendMessage(frames[0], "click link");
    assert_equals(child_loaded2, "loaded");
    const child_base_uri2 = await sendMessage(frames[0], "report baseURI");
    assert_not_equals(child_base_uri2, non_default_parent_baseURI);
    assert_not_equals(child_base_uri2, original_parent_baseURI);
    assert_equals(child_base_uri2, nav_target);

    // Parent resets its baseURI.
    base_element.remove();
    assert_equals(document.baseURI, original_parent_baseURI,
        "parent baseURI failed to reset.");

    // Navigate child back and verify its baseURI didn't change.
    const child_loaded3 = new Promise(r => onmessage = e => r(e.data));
    window.history.back();
    assert_equals(await child_loaded3, "loaded");
    const child_base_uri3 = await sendMessage(frames[0], "report baseURI");
    assert_equals(child_base_uri3, non_default_parent_baseURI);
  }, description);
}

runTest("non-sandboxed srcdoc - parent changes baseURI",null);
runTest("sandboxed srcdoc - parent changes baseURI", "allow-scripts");
