// Reading baseURI cannot distinguish a snapshot from a live lookup on its own, as
// implementations cache the base URL. So change the container's base URL and then
// force the child to recompute by inserting and removing a <base>.
//
// See https://github.com/whatwg/dom/issues/454

function makeIframe(doc, configure) {
  return new Promise(resolve => {
    const iframe = doc.createElement("iframe");
    configure(iframe);
    iframe.addEventListener("load", () => resolve(iframe), { once: true });
    doc.body.appendChild(iframe);
  });
}

function forceBaseURLRecompute(doc) {
  const base = doc.createElement("base");
  base.setAttribute("href", "http://example.org/forced/");
  doc.head.appendChild(base);
  doc.head.removeChild(base);
}

function notLiveTest(configureChild) {
  return async t => {
    const container = await makeIframe(document, f => f.src = "about:blank");
    t.add_cleanup(() => container.remove());
    const cdoc = container.contentDocument;

    const child = await makeIframe(cdoc, configureChild);
    const childDoc = child.contentDocument;
    const inherited = childDoc.baseURI;

    const cbase = cdoc.createElement("base");
    cbase.setAttribute("href", "http://example.com/changed/");
    cdoc.head.appendChild(cbase);
    assert_equals(cdoc.baseURI, "http://example.com/changed/", "container base URL changed");

    forceBaseURLRecompute(childDoc);
    assert_equals(childDoc.baseURI, inherited,
      "child base URL is a snapshot from creation, not the container's current base URL");
  };
}

promise_test(notLiveTest(f => f.src = "about:blank"), "about:blank base URL is not live");

promise_test(notLiveTest(f => f.srcdoc = "x"), "about:srcdoc base URL is not live");
