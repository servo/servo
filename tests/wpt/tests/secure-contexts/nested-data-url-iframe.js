// Tests the secure context state of documents that are nested inside, or
// opened by, a data: URL iframe.
//
// Per the HTML Standard, whether a window environment is a secure context
// depends on its top-level creation URL. For every case below, that is this
// (top-level) test document's URL: nested documents share it as their
// top-level document, and a popup's initial about:blank inherits its
// creator's top-level creation URL. So each case must be a secure context iff
// this document is.
//
// https://html.spec.whatwg.org/multipage/webappapis.html#secure-context
// https://html.spec.whatwg.org/multipage/document-sequences.html#creating-a-new-browsing-context

const reporter = id => `<script>top.postMessage({id: ${JSON.stringify(id)}, ` +
    `secure: isSecureContext}, "*");<\/script>`;

function awaitReport(id) {
  return new Promise(resolve => {
    window.addEventListener('message', function handler(e) {
      if (e.data && e.data.id === id) {
        window.removeEventListener('message', handler);
        resolve(e.data.secure);
      }
    });
  });
}

// Serializes `s` as a JS string literal that is safe to embed in an inline
// script element (i.e., it cannot contain a literal closing script tag).
const jsString = s => JSON.stringify(s).replace(/</g, '\\u003c');

const dataUrlWithScript = script =>
    'data:text/html,' + encodeURIComponent(`<body><script>${script}<\/script>`);

const dataUrlContainingIframe = childSetup => dataUrlWithScript(`
  const f = document.createElement("iframe");
  ${childSetup}
  document.body.appendChild(f);`);

const cases = [
  [
    'a data: URL iframe',
    id => 'data:text/html,' + encodeURIComponent(reporter(id))
  ],
  [
    'a srcdoc iframe nested in a data: URL iframe',
    id => dataUrlContainingIframe(`f.srcdoc = ${jsString(reporter(id))};`)
  ],
  [
    'a data: URL iframe nested in a data: URL iframe',
    id => dataUrlContainingIframe(`f.src = ${
        jsString('data:text/html,' + encodeURIComponent(reporter(id)))};`)
  ],
  [
    'a sandboxed srcdoc iframe nested in a data: URL iframe',
    id => dataUrlContainingIframe(`f.sandbox = "allow-scripts";
                                  f.srcdoc = ${jsString(reporter(id))};`)
  ],
  [
    'the initial about:blank of a popup opened by a data: URL iframe',
    id => dataUrlWithScript(`
     const w = window.open();
     top.postMessage({id: ${jsString(id)}, secure: w.isSecureContext}, "*");
     w.close();`)
  ],
];

for (const [desc, makeUrl] of cases) {
  promise_test(async t => {
    const id = desc;
    const iframe = document.createElement('iframe');
    t.add_cleanup(() => iframe.remove());
    const result = awaitReport(id);
    iframe.src = makeUrl(id);
    document.body.appendChild(iframe);
    assert_equals(await result, window.isSecureContext,
                  'should be a secure context iff the top-level document is');
  }, `isSecureContext in ${desc}`);
}
