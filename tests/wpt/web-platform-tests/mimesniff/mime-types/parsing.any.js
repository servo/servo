// META: timeout=long

promise_test(() => {
  return Promise.all([
    fetch("resources/mime-types.json"),
    fetch("resources/generated-mime-types.json")
  ]).then(([res, res2]) => res.json().then(runTests).then(() => res2.json().then(runTests)));
}, "Loading data…");

function isByteCompatible(str) {
  for(let i = 0; i < str.length; i++) {
    const charCode = str.charCodeAt(i);
    // See https://fetch.spec.whatwg.org/#concept-header-value
    if(charCode > 0xFF) {
      return "incompatible";
    } else if(charCode === 0x00 || charCode === 0x0A || charCode === 0x0D) {
      return "header-value-incompatible";
    }
  }
  return "compatible";
}

function runTests(tests) {
  tests.forEach(val => {
    if(typeof val === "string") {
      return;
    }
    const output = val.output === null ? "" : val.output
    test(() => {
      assert_equals(new Blob([], { type: val.input}).type, output, "Blob");
      assert_equals(new File([], "noname", { type: val.input}).type, output, "File");
    }, val.input + " (Blob/File)");

    promise_test(() => {
      const compatibleNess = isByteCompatible(val.input);
      if(compatibleNess === "incompatible" || compatibleNess === "header-value-incompatible") {
        assert_throws_js(TypeError, () => new Request("about:blank", { headers: [["Content-Type", val.input]] }));
        assert_throws_js(TypeError, () => new Response(null, { headers: [["Content-Type", val.input]] }));
        return Promise.resolve();
      } else {
        return Promise.all([
          new Request("about:blank", { headers: [["Content-Type", val.input]] }).blob().then(blob => assert_equals(blob.type, output)),
          new Response(null, { headers: [["Content-Type", val.input]] }).blob().then(blob => assert_equals(blob.type, output))
        ]);
      }
    }, val.input + " (Request/Response)");
  });
}
