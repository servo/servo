["a",
 "area",
 "link"].forEach(type => {
  async_test(t => {
    const frame = document.createElement("iframe"),
          link = document.createElement(type);
    t.add_cleanup(() => frame.remove());
    frame.onload = t.step_func(() => {
      // See https://github.com/whatwg/html/issues/490
      if(frame.contentWindow.location.href === "about:blank")
        return;
      link.click(); // must be ignored because document is not active
      t.step_timeout(() => {
        assert_equals(frame.contentWindow.location.pathname, "/common/blank.html");
        t.done();
      }, 500);
    });
    document.body.appendChild(frame);
    frame.contentDocument.body.appendChild(link);
    link.href = "/";
    frame.src = "/common/blank.html";
  }, "<" + type + "> in navigated away <iframe>'s document cannot follow hyperlinks");
});
