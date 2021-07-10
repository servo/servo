// META: script=/common/get-host-info.sub.js

[
  "frame", // This works without <frameset>, so great
  "iframe",
  "object",
  "embed",
].forEach(element => {
  [
    null,
    "",
    "initialvalue"
  ].forEach(initialNameValue => {
    [
      "same-origin",
      "cross-origin"
    ].forEach(originType => {
      async_test(t => {
        const ident = element + initialNameValue + originType,
              file = `${new URL("resources/post-to-parent.html", location.href).pathname}?ident=${ident}`,
              child = originType === "same-origin" ? file : `${get_host_info().HTTP_REMOTE_ORIGIN}${file}`,
              frame = document.createElement(element),
              expectedNameValue = initialNameValue || "";
        let state = "set";
        const listener = t.step_func(e => {
          if (e.data.ident === ident) {
            assert_equals(e.data.name, expectedNameValue); // This check is always the same
            if (state === "set") {
              frame.setAttribute("name", "meh");
              state = "remove"
              e.source.postMessage(null, "*");
              return;
            }
            if (state === "remove") {
              frame.removeAttribute("name");
              state = "done";
              e.source.postMessage(null, "*");
              return;
            }
            if (state === "done") {
              t.done();
            }
          }
        });
        frame.setAttribute(element === "object" ? "data" : "src", child);
        if (initialNameValue !== null) {
          frame.setAttribute("name", initialNameValue);
        }
        t.add_cleanup(() => {
          self.removeEventListener("message", listener);
          frame.remove();
        });
        self.addEventListener("message", listener);
        document.body.append(frame);
      }, `${originType} <${element}${initialNameValue !== null ? ' name=' + initialNameValue : ''}>`);
    });
  });
});
