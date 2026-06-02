// META: script=/common/get-host-info.sub.js
// Test that the initial about:blank gets the about base URL
// from the entry global object.

async function withIframe(src, cb) {
    const ifr = document.createElement("iframe");
    ifr.src = src;
    document.body.append(ifr);
    cb(ifr);
    ifr.remove();
}

async function withWindow(src, cb) {
    const w = window.open(src);
    cb(w);
    w.close();
}

// Need a trailing '/' for equality checks
const REMOTE_ORIGIN = new URL("/", get_host_info().REMOTE_ORIGIN).href;

async function withWindowOpenerNotInitiator(src, cb) {
    window.deferredIframeWindow = Promise.withResolvers();

    // Create an iframe with a different base URL.
    // If it opens a window with window.top being the opener,
    // the base URL should come from the initiator, i.e. this iframe.
    const ifr = document.createElement("iframe");
    ifr.srcdoc = `
    <head>
    <base href='${REMOTE_ORIGIN}'>
    <script>
        const w = window.top.open('${src}');
        window.top.deferredIframeWindow.resolve(w);
    </scr` + `ipt>
    </head>
    <body></body>
    `;
    document.body.append(ifr);

    const w = await window.deferredIframeWindow.promise;

    cb(w);

    w.close();
    ifr.remove();
}

promise_test(async t => {
    await withIframe("", ifr => {
        assert_equals(ifr.contentDocument.baseURI, document.baseURI, "about:blank has creator's base URI");
    })
}, "Initial iframe about:blank gets base url from creator");

promise_test(async t => {
    await withIframe("/arbitrary-sameorigin-src", ifr => {
        assert_equals(ifr.contentDocument.baseURI, document.baseURI, "about:blank has creator's base URI");
    })
}, "Transient iframe about:blank gets base url from creator");

promise_test(async t => {
    await withWindow("", w => {
        assert_equals(w.document.baseURI, document.baseURI, "about:blank has creator's base URI");
    })
}, "Initial top-level about:blank gets base url from creator = opener");

promise_test(async t => {
    await withWindow("/arbitrary-sameorigin-src", w => {
        assert_equals(w.document.baseURI, document.baseURI, "about:blank has creator's base URI");
    })
}, "Transient top-level about:blank gets base url from creator = opener");

promise_test(async t => {
    await withWindowOpenerNotInitiator("", w => {
        assert_not_equals(REMOTE_ORIGIN, document.baseURI, "These need to be different");
        assert_equals(w.document.baseURI, REMOTE_ORIGIN, "about:blank has creator's base URI");
    })
}, "Initial top-level about:blank gets base url from creator != opener");

promise_test(async t => {
    await withWindowOpenerNotInitiator("/arbitrary-sameorigin-src", w => {
        assert_not_equals(REMOTE_ORIGIN, document.baseURI, "These need to be different");
        assert_equals(w.document.baseURI, REMOTE_ORIGIN, "about:blank has creator's base URI");
    })
}, "Transient top-level about:blank gets base url from creator != opener");
