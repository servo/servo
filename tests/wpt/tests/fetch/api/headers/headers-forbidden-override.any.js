// META: global=window,worker

let forbiddenMethods = [
  "TRACE",
  "TRACK",
  "CONNECT",
  "trace",
  "track",
  "connect",
  "\rtrace",
  "\ttrack",
  "\nconnect",
  "trace,",
  "GET,track ",
  " connect",
];

let overrideHeaders = [
  "x-http-method-override",
  "x-http-method",
  "x-method-override",
  "X-HTTP-METHOD-OVERRIDE",
  "X-HTTP-METHOD",
  "X-METHOD-OVERRIDE",
];

for (forbiddenMethod of forbiddenMethods) {
  for (overrideHeader of overrideHeaders) {
    test(() => {
      let r = new Request("https://site.example/");
      r.headers.append(overrideHeader, forbiddenMethod);
      assert_false(r.headers.has(overrideHeader));
    }, `header ${overrideHeader} is forbidden to use value ${forbiddenMethod}`);
  }
}

let permittedValues = [
  "GETTRACE",
  "GET",
  "\",TRACE\",",
];

for (permittedValue of permittedValues) {
    for (overrideHeader of overrideHeaders) {
      test(() => {
        let r = new Request("https://site.example/");
        r.headers.append(overrideHeader, permittedValue);
        assert_equals(permittedValue, r.headers.get(overrideHeader));
      }, `header ${overrideHeader} is allowed to use value ${permittedValue}`);
    }
}
