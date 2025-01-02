// META: script=/common/get-host-info.sub.js

// Load about:srcdoc in a sandboxed iframe. Check the document.baseURI is
// correct.
const runTest = (description, iframe_sandbox) => {
  promise_test(async test => {
    const iframe = document.createElement("iframe");
    iframe.sandbox = iframe_sandbox;
    iframe.srcdoc = `
      <script>
        parent.postMessage(document.baseURI, '*');
      </scr`+`ipt>
    `;
    const child_base_uri = new Promise(r => onmessage = e => r(e.data));
    document.body.appendChild(iframe);
    // [spec]: https://html.spec.whatwg.org/C/#fallback-base-url
    // Step 1: If document is an iframe srcdoc document, then return the
    //         document base URL of document's browsing context's container
    //         document.
    assert_equals(await child_base_uri, document.baseURI);
  }, description);
}

onload = () => {
  runTest("allow-same-origin", "allow-scripts allow-same-origin");
  runTest("disallow-same-origin", "allow-scripts");
}
