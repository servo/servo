["<link>", "@import"].forEach(linkType => {
  [
   ["same-origin", "resources/css.py"],

   // TODO(lukasza@chromium.org): Once https://crbug.com/888079 and
   // https://crbug.com/891872 are fixed, we should use a cross-*origin* rather
   // than cross-*site* URL below (e.g.  s/ HTTP_NOTSAMESITE_ORIGIN /
   // HTTP_REMOTE_ORIGIN /g).  See also https://crbug.com/918660 for more
   // context.
   ["cross-origin", get_host_info().HTTP_NOTSAMESITE_ORIGIN + "/html/semantics/document-metadata/the-link-element/resources/css.py"]
  ].forEach(originType => {
    ["no Content-Type", "wrong Content-Type", "broken Content-Type"].forEach(contentType => {
      ["no nosniff", "nosniff"].forEach(nosniff => {
        async_test(t => {
          const l = document.createElement("link");
          t.add_cleanup(() => l.remove());
          if (nosniff === "nosniff" || contentType === "wrong Content-Type" && (document.compatMode === "CSS1Compat" || originType[0] === "cross-origin")) {
            l.onerror = t.step_func_done();
            l.onload = t.unreached_func("error event should have fired");
          } else {
            l.onload = t.step_func_done();
            l.onerror = t.unreached_func("load event should have fired");
          }
          l.rel = "stylesheet";
          let query = [];
          if (contentType === "broken Content-Type") {
            query.push("content_type=oops");
          } else if (contentType === "wrong Content-Type") {
            query.push("content_type=text/plain")
          }
          if (nosniff === "nosniff") {
            query.push("nosniff");
          }
          let stringQuery = "";
          query.forEach(val => {
            if (stringQuery === "") {
              stringQuery += "?" + val;
            } else {
              stringQuery += "&" + val;
            }
          });
          const link = new URL(originType[1] + stringQuery, location).href;
          if (linkType === "<link>") {
            l.href = link;
          } else {
            l.href = "data:text/css,@import url(" + link + ");";
          }
          document.head.appendChild(l);
        }, "Stylesheet loading using " + linkType + " with " + contentType + ", " + originType[0] + ", and " + nosniff);
      });
    });
  });
});
