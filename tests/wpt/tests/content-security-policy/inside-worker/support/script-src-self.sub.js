importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

let importscripts_url ="https://{{hosts[][www]}}:{{ports[https][1]}}" +
    "/content-security-policy/support/var-a.js";

promise_test(async t => {
  self.a = false;
  assert_throws_dom("NetworkError",
                    _ => importScripts(importscripts_url),
                    "importScripts should throw `NetworkError`");
  assert_false(self.a);
  return waitUntilCSPEventForURL(t, importscripts_url);
}, "Cross-origin `importScripts()` blocked in " + self.location.protocol +
             " with {{GET[test-name]}}");

promise_test(t => {
  assert_throws_js(EvalError,
                   _ => eval("1 + 1"),
                   "`eval()` should throw 'EvalError'.");

  assert_throws_js(EvalError,
                   _ => new Function("1 + 1"),
                   "`new Function()` should throw 'EvalError'.");
  return Promise.all([
    waitUntilCSPEventForEval(t, 19),
    waitUntilCSPEventForEval(t, 23),
  ]);
}, "`eval()` blocked in " + self.location.protocol +
             " with {{GET[test-name]}}");

promise_test(t => {
  self.setTimeoutTest = t;
  let result = setTimeout("(self.setTimeoutTest.unreached_func(" +
                          "'setTimeout([string]) should not execute.'))()", 1);
  assert_equals(result, 0);
  return waitUntilCSPEventForEval(t, 34);
}, "`setTimeout([string])` blocked in " + self.location.protocol +
             " with {{GET[test-name]}}");

promise_test(async t => {
  let report_url = "{{location[server]}}/reporting/resources/report.py" +
      "?op=retrieve_report&reportID={{GET[id]}}&min_count=4";

  let response = await fetch(report_url);
  assert_equals(response.status, 200, "Fetching reports failed");

  let response_json = await response.json();
  let reports = response_json.map(x => x["csp-report"]);

  assert_array_equals(
      reports.map(x => x["blocked-uri"]).sort(),
      [ importscripts_url, "eval", "eval", "eval" ].sort(),
      "Reports do not match");
  assert_array_equals(
      reports.map(x => x["violated-directive"]).sort(),
      [ "script-src-elem", "script-src", "script-src", "script-src" ].sort(),
      "Violated directive in report does not match");
  assert_array_equals(
      reports.map(x => x["effective-directive"]).sort(),
      [ "script-src-elem", "script-src", "script-src", "script-src" ].sort(),
      "Effective directive in report does not match");
  reports.forEach(x => {
    assert_equals(
        x["disposition"], "enforce",
        "Disposition in report does not match");
  });
}, "Reports are sent for " + self.location.protocol +
                  " with {{GET[test-name]}}");

done();
