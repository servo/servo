for (const [ev, target] of [
  ["beforeunload", iframe => iframe.contentWindow],
  ["pagehide", iframe => iframe.contentWindow],
  ["unload", iframe => iframe.contentWindow],
  ["visibilitychange", iframe => iframe.contentDocument],
]) {
  async_test(t => {
    const iframe = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => iframe.remove());
    iframe.src = "/common/blank.html";
    iframe.onload = t.step_func(() => {
      target(iframe).addEventListener(ev, t.step_func_done(() => {
        assert_not_equals(iframe.contentDocument.childNodes.length, 0);
        assert_equals(iframe.contentDocument.open(), iframe.contentDocument);
        assert_not_equals(iframe.contentDocument.childNodes.length, 0);
      }));
      iframe.src = "about:blank";
    });
  }, `document.open should bail out when ignore-opens-during-unload is greater than 0 during ${ev} event (in top-level browsing context)`);

  async_test(t => {
    const iframe = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => iframe.remove());
    iframe.src = "/common/blank.html?1";
    iframe.onload = t.step_func(() => {
      const doc = iframe.contentDocument;
      const innerIframe = doc.body.appendChild(doc.createElement("iframe"));
      innerIframe.src = "/common/blank.html?2";
      innerIframe.onload = t.step_func(() => {
        // Navigate the parent, listen on the child, and open() the parent.
        target(innerIframe).addEventListener(ev, t.step_func_done(() => {
          assert_not_equals(iframe.contentDocument.childNodes.length, 0);
          iframe.contentDocument.open();
          assert_not_equals(iframe.contentDocument.childNodes.length, 0);
        }));
        iframe.src = "about:blank";
      });
    });
  }, `document.open should bail out when ignore-opens-during-unload is greater than 0 during ${ev} event (open(parent) while unloading parent and child)`);

  async_test(t => {
    const iframe = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => iframe.remove());
    iframe.src = "/common/blank.html?1";
    iframe.onload = t.step_func(() => {
      const doc = iframe.contentDocument;
      const innerIframe = doc.body.appendChild(doc.createElement("iframe"));
      innerIframe.src = "/common/blank.html?2";
      innerIframe.onload = t.step_func(() => {
        // Navigate the child, listen on the child, and open() the parent.
        target(innerIframe).addEventListener(ev, t.step_func_done(() => {
          assert_not_equals(iframe.contentDocument.childNodes.length, 0);
          iframe.contentDocument.open();
          assert_equals(iframe.contentDocument.childNodes.length, 0);
        }));
        innerIframe.src = "about:blank";
      });
    });
  }, `document.open should bail out when ignore-opens-during-unload is greater than 0 during ${ev} event (open(parent) while unloading child only)`);
}
