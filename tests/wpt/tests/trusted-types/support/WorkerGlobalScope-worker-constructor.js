const test_setup_policy = trustedTypes.createPolicy("p",
  { createScriptURL: s => s }
);

importScripts(test_setup_policy.createScriptURL("/resources/testharness.js"));

test(() => {
  assert_throws_js(TypeError, () => { new Worker("w"); },
    "Creating a Worker threw");
}, "Creating a Worker from a string should throw");

test(() => {
  new Worker(test_setup_policy.createScriptURL("u"));
}, "Creating a Worker from a TrustedScriptURL should not throw");

test(() => {
  trustedTypes.createPolicy("default",
    { createScriptURL: s => "defaultValue" });

  new Worker("s");
}, "Creating a Worker from a string with a default policy should not throw");

done();
