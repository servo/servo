let test_setup_policy = trustedTypes.createPolicy("hurrayanythinggoes", {
  createScriptURL: x => x
});
importScripts(test_setup_policy.createScriptURL("/resources/testharness.js"));

// Determine worker type (for better logging)
let worker_type = "unknown";
if (this.DedicatedWorkerGlobalScope !== undefined) {
  worker_type = "dedicated worker";
} else if (this.SharedWorkerGlobalScope !== undefined) {
  worker_type = "shared worker";
} else if (this.ServiceWorkerGlobalScope !== undefined) {
  worker_type = "service worker";
}

let test_policy = trustedTypes.createPolicy("xxx", {
  createScriptURL: url => url.replace("play", "work")
});

test(t => {
  self.result = "Fail";
  let trusted_url = test_policy.createScriptURL("player.js");
  assert_true(this.trustedTypes.isScriptURL(trusted_url));
  importScripts(trusted_url);  // worker.js modifies self.result.
  assert_equals(self.result, "Pass");
}, "importScripts with TrustedScriptURL works in " + worker_type);

test(t => {
  let untrusted_url = "player.js";
  assert_throws_js(TypeError,
    function() { importScripts(untrusted_url) },
    "importScripts(untrusted_url)");
}, "importScripts with untrusted URLs throws in " + worker_type);

test(t => {
  assert_throws_js(TypeError,
    function() { importScripts(null) },
    "importScripts(null)");
}, "null is not a trusted script URL throws in " + worker_type);

test(t => {
  self.result = "Fail";
  let trusted_url = test_policy.createScriptURL("player.js?variant1");
  let trusted_url2 = test_policy.createScriptURL("player.js?variant2");
  importScripts(trusted_url, trusted_url2);
  assert_equals(self.result, "Pass");
}, "importScripts with two URLs, both trusted, in " + worker_type);

test(t => {
  let untrusted_url = "player.js?variant1";
  let untrusted_url2 = "player.js?variant2";
  assert_throws_js(TypeError,
    function() { importScripts(untrusted_url, untrusted_url2) },
    "importScripts(untrusted_url, untrusted_url2)");
}, "importScripts with two URLs, both strings, in " + worker_type);

test(t => {
  let untrusted_url = "player.js";
  let trusted_url = test_policy.createScriptURL(untrusted_url);
  assert_throws_js(TypeError,
    function() { importScripts(untrusted_url, trusted_url) },
    "importScripts(untrusted_url, trusted_url)");
}, "importScripts with two URLs, one trusted, in " + worker_type);

// Test default policy application:
trustedTypes.createPolicy("default", {
  createScriptURL: (url, _, sink) => {
    assert_equals(sink, "Worker importScripts");
    return url.replace("play", "work");
  }
}, true);
test(t => {
  self.result = "Fail";
  let untrusted_url = "player.js";
  importScripts(untrusted_url);
  assert_equals(self.result, "Pass");
}, "importScripts with untrusted URLs and default policy works in " + worker_type);

test(t => {
  self.result = "Fail";
  let untrusted_url = "player.js";
  let trusted_url = test_policy.createScriptURL(untrusted_url);
  importScripts(untrusted_url, trusted_url);
  assert_equals(self.result, "Pass");
}, "importScripts with one trusted and one untrusted URLs and default policy works in " + worker_type);

done();
