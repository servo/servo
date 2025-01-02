// META: script=/common/get-host-info.sub.js

function echoURL(content) {
  return `/common/echo.py?content=${encodeURIComponent(content)}`;
}

function setSrc(frame, type, content) {
  if (type === "same-origin") {
    frame.src = echoURL(content);
  } else if (type === "cross-site") {
    frame.src = `${get_host_info().HTTP_NOTSAMESITE_ORIGIN}${echoURL(content)}`;
  } else {
    frame.srcdoc = content;
  }
}

["srcdoc", "same-origin", "cross-site"].forEach(type => {
  const initialType = type === "srcdoc" ? type : "same-origin";

  [
    {
      "namedObject": "<div id=abc></div>",
      "namedObjectLocalName": "div"
    },
    {
      "namedObject": "<object name=abc></object>",
      "namedObjectLocalName": "object"
    },
    {
      "namedObject": "<iframe id=abc></iframe>",
      "namedObjectLocalName": "iframe"
    }
  ].forEach(testData => {
    async_test(t => {
      const frame = document.createElement("iframe");
      t.add_cleanup(() => frame.remove());
      setSrc(frame, initialType, `<script>function f() { return abc }</script>${testData.namedObject}`);
      frame.onload = t.step_func(() => {
        const f = frame.contentWindow.f,
              associatedAbc = f();
        frame.onload = t.step_func_done(() => {
          assert_equals(f(), associatedAbc);
          assert_equals(associatedAbc.localName, testData.namedObjectLocalName);
        });
        setSrc(frame, type, "<span id=abc></span>");
      });
      document.body.append(frame);
    }, `Window's associated Document object is used for finding named objects (<${testData.namedObjectLocalName}> via ${type} <iframe>)`);
  });

  async_test(t => {
    const frame = document.createElement("iframe");
    t.add_cleanup(() => frame.remove());
    setSrc(frame, initialType, "<script>function f() { return abc }</script><object name=abc data='about:blank'></object>");
    frame.onload = t.step_func(() => {
      const f = frame.contentWindow.f,
            associatedAbc = f(),
            associatedAbcContainer = associatedAbc.frameElement;
      frame.onload = t.step_func_done(() => {
        assert_equals(f(), associatedAbcContainer);
        assert_equals(associatedAbcContainer.contentWindow, null);
      });
      setSrc(frame, type, "<span id=abc></span>");
    });
    document.body.append(frame);
  }, `Window's associated Document object is used for finding named objects (<object> with browsing ccontext via ${type} <iframe)>`);
});
