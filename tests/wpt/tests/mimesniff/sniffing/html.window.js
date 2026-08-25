["atom", "rss"].forEach(item => {
  async_test(t => {
    const popup = window.open(`support/${item}.html`);
    t.add_cleanup(() => popup.close());
    popup.onload = t.step_func_done(() => {
      assert_equals(popup.document.contentType, "text/html");
      assert_equals(popup.document.documentElement.localName, "html");
      assert_equals(popup.document.documentElement.namespaceURI, "http://www.w3.org/1999/xhtml");
      assert_equals(popup.document.querySelector("b").namespaceURI, "http://www.w3.org/1999/xhtml");
    });
  }, `HTML is not sniffed for a "feed": ${item}`);
});
