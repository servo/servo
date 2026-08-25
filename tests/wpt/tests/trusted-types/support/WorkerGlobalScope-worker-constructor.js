const test_setup_policy = trustedTypes.createPolicy("p",
  { createScriptURL: s => s }
);

importScripts(test_setup_policy.createScriptURL("/resources/testharness.js"));

// Determine worker type (for better logging)
let worker_type = "unknown";
if (this.DedicatedWorkerGlobalScope !== undefined) {
  worker_type = "dedicated worker";
} else if (this.SharedWorkerGlobalScope !== undefined) {
  worker_type = "shared worker";
}

test(() => {
  assert_throws_js(TypeError, () => { new Worker("w"); },
    "Creating a Worker threw");
}, `Creating a Worker from a string should throw (${worker_type} scope)`);

test(() => {
  new Worker(test_setup_policy.createScriptURL("u"));
}, `Creating a Worker from a TrustedScriptURL should not throw (${worker_type} scope)`);

test(() => {
  trustedTypes.createPolicy("default",
    { createScriptURL: (s, _, sink) => {
        assert_equals(sink, 'Worker constructor');
        return "defaultValue";
      }
    }
  );

  new Worker("s");
}, `Creating a Worker from a string with a default policy should not throw (${worker_type} scope)`);

done();
