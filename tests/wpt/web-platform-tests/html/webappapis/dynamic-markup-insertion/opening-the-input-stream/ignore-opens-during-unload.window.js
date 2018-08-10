for (const ev of ["unload", "beforeunload", "pagehide"]) {
  async_test(t => {
    const iframe = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => iframe.remove());
    iframe.src = "/common/blank.html";
    iframe.onload = t.step_func(() => {
      iframe.contentWindow.addEventListener(ev, t.step_func_done(() => {
        assert_not_equals(iframe.contentDocument.childNodes.length, 0);
        assert_equals(iframe.contentDocument.open(), iframe.contentDocument);
        assert_not_equals(iframe.contentDocument.childNodes.length, 0);
      }));
      iframe.src = "about:blank";
    });
  }, `document.open should bail out when ignore-opens-during-unload is greater than 0 during ${ev} event`);
}
