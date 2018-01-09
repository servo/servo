promise_test(() => {
  // Don't load generated-mime-types.json as none of them are navigable
  return fetch("resources/mime-types.json").then(res => res.json().then(runTests));
}, "Loading data…");

function isByteCompatible(str) {
  for(let i = 0; i < str.length; i++) {
    const charCode = str.charCodeAt(i);
    // See https://github.com/w3c/web-platform-tests/issues/8372 for 0x0B and 0x0C
    // See https://fetch.spec.whatwg.org/#concept-header-value for the remainder
    if(charCode > 0xFF) {
      return "incompatible";
    } else if(charCode === 0x00 || charCode === 0x0A || charCode === 0x0D) {
      return "header-value-incompatible";
    } else if(charCode === 0x0B || charCode === 0x0C) {
      return "wptserve-incompatible";
    }
  }
  return "compatible";
}

function encodeForURL(str) {
  let output = "";
  for(let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    if(char > 0xFF) {
      throw new Error("We cannot deal with input that is not latin1");
    } else  {
      output += "%" + char.toString(16).padStart(2, "0");
    }
  }
  return output;
}

function runTests(tests) {
  tests.forEach(val => {
    if(typeof val === "string" || val.navigable === undefined || val.encoding === undefined || isByteCompatible(val.input) !== "compatible") {
      return;
    }
    const mime = val.input;
    async_test(t => {
      const frame = document.createElement("iframe"),
            expectedEncoding = val.encoding === null ? "UTF-8" : val.encoding;
      t.add_cleanup(() => frame.remove());
      frame.onload = t.step_func(() => {
        if(frame.contentWindow.location.href === "about:blank") {
          return;
        }
        // Edge fails all these tests due to not using the correct encoding label.
        assert_equals(frame.contentDocument.characterSet, expectedEncoding);
        t.done();
      });
      frame.src = "resources/mime-charset.py?type=" + encodeForURL(mime);
      document.body.appendChild(frame);
    }, mime);
  });
}
