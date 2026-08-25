document.domain = "{{host}}";

// In many cases in this test, we want to delay execution of a piece of code so
// that the entry settings object would be the top-level page. A microtask is
// perfect for this purpose as it is executed in the "clean up after running
// script" algorithm, which is generally called right after the callback.
function setEntryToTopLevel(cb) {
  Promise.resolve().then(cb);
}

async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { iframe.remove(); });
  iframe.onload = t.step_func_done(() => {
    // Since this is called as an event handler on an element of this window,
    // the entry settings object is that of this browsing context.
    assert_throws_dom(
      "InvalidStateError",
      iframe.contentWindow.DOMException,
      () => {
        iframe.contentDocument.open();
      },
      "opening an XML document should throw an InvalidStateError"
    );
  });
  const frameURL = new URL("resources/bailout-order-xml-with-domain-frame.sub.xhtml", document.URL);
  frameURL.port = "{{ports[http][1]}}";
  iframe.src = frameURL.href;
}, "document.open should throw an InvalidStateError with XML document even if it is cross-origin");

async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { iframe.remove(); });
  window.onCustomElementReady = t.step_func(() => {
    window.onCustomElementReady = t.unreached_func("onCustomElementReady called again");
    // Here, the entry settings object is still the iframe's, as the function
    // is called from a custom element constructor in the iframe document.
    // Delay execution in such a way that makes the entry settings object the
    // top-level page's, but without delaying too much that the
    // throw-on-dynamic-markup-insertion counter gets decremented (which is
    // what this test tries to pit against the cross-origin document check).
    //
    // "Clean up after running script" is executed through the "construct" Web
    // IDL algorithm in "create an element", called by "create an element for a
    // token" in the parser.
    setEntryToTopLevel(t.step_func_done(() => {
      assert_throws_dom(
        "InvalidStateError",
        iframe.contentWindow.DOMException,
        () => {
          iframe.contentDocument.open();
        },
        "opening a document when the throw-on-dynamic-markup-insertion counter is incremented should throw an InvalidStateError"
      );
    }));
  });
  const frameURL = new URL("resources/bailout-order-custom-element-with-domain-frame.sub.html", document.URL);
  frameURL.port = "{{ports[http][1]}}";
  iframe.src = frameURL.href;
}, "document.open should throw an InvalidStateError when the throw-on-dynamic-markup-insertion counter is incremented even if the document is cross-origin");

async_test(t => {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { iframe.remove(); });
  self.testSynchronousScript = t.step_func(() => {
    // Here, the entry settings object is still the iframe's, as the function
    // is synchronously called from a <script> element in the iframe's
    // document.
    //
    // "Clean up after running script" is executed when the </script> tag is
    // seen by the HTML parser.
    setEntryToTopLevel(t.step_func_done(() => {
      assert_throws_dom(
        "SecurityError",
        iframe.contentWindow.DOMException,
        () => {
          iframe.contentDocument.open();
        },
        "opening a same origin-domain (but not same origin) document should throw a SecurityError"
      );
    }));
  });
  const frameURL = new URL("resources/bailout-order-synchronous-script-with-domain-frame.sub.html", document.URL);
  frameURL.port = "{{ports[http][1]}}";
  iframe.src = frameURL.href;
}, "document.open should throw a SecurityError with cross-origin document even when there is an active parser executing script");

for (const ev of ["beforeunload", "pagehide", "unload"]) {
  async_test(t => {
    const iframe = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => { iframe.remove(); });
    iframe.addEventListener("load", t.step_func(() => {
      iframe.contentWindow.addEventListener(ev, t.step_func(() => {
        // Here, the entry settings object should be the top-level page's, as
        // the callback context of this event listener is the incumbent
        // settings object, which is the this page. However, due to a Chrome
        // bug (https://crbug.com/606900), the entry settings object may be
        // mis-set to the iframe's.
        //
        // "Clean up after running script" is called in the task that
        // navigates.
        setEntryToTopLevel(t.step_func_done(() => {
          assert_throws_dom(
            "SecurityError",
            iframe.contentWindow.DOMException,
            () => {
              iframe.contentDocument.open();
            },
            "opening a same origin-domain (but not same origin) document should throw a SecurityError"
          );
        }));
      }));
      iframe.src = "about:blank";
    }), { once: true });
    iframe.src = "http://{{host}}:{{ports[http][1]}}/common/domain-setter.sub.html";
  }, `document.open should throw a SecurityError with cross-origin document even when the ignore-opens-during-unload counter is greater than 0 (during ${ev} event)`);
}
