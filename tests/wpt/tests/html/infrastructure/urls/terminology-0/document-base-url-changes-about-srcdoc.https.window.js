// Load about:srcdoc in a iframe. Check the document.baseURI is still
// correct after the parent changes its base URL..
const runTest = (description, sandbox_flags) => {
  promise_test(async test => {
    // Create child.
    const iframe = document.createElement("iframe");
    if (sandbox_flags !== null)
      iframe.sandbox = sandbox_flags;
    iframe.srcdoc = `
      <script>
        addEventListener('message', (event) => {
          if (event.data == 'report baseURI')
            event.source.postMessage(document.baseURI, event.origin);
        });
        parent.postMessage('loaded', '*');
      </scr`+`ipt>
    `;

    const child_loaded = new Promise(r => onmessage = e => r(e.data));
    document.body.appendChild(iframe);
    assert_equals(await child_loaded, "loaded");

    // Verify child's baseURI matches parent.
    const original_parent_baseURI = document.baseURI;
    const child_base_uri = new Promise(r => onmessage = e => r(e.data));
    frames[0].postMessage("report baseURI", "*");
    assert_equals(await child_base_uri, original_parent_baseURI);

    // Parent changes its baseURI, requests child to report.
    const base_element = document.createElement("base");
    base_element.href = "https://foo.com";
    document.head.appendChild(base_element);
    assert_not_equals(document.baseURI, original_parent_baseURI,
        "parent baseURI failed to change.");

    // Verify child's baseURI didn't change.
    const child_base_uri2 = new Promise(r => onmessage = e => r(e.data));
    frames[0].postMessage("report baseURI", "*");
    assert_equals(await child_base_uri2, original_parent_baseURI);

    // Cleanup.
    base_element.remove();
  }, description);
}

runTest("non-sandboxed srcdoc - parent changes baseURI",null);
runTest("sandboxed srcdoc - parent changes baseURI", "allow-scripts");
