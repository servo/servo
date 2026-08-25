// META: global=window,dedicatedworker,sharedworker

for (const value of [null, true, false, "string"]) {
  promise_test(async t => {
    const result = await import(`./${value}.json`, { with: { type: "json" } });
    assert_equals(result.default, value);
  }, `Non-object: ${value}`);
}

promise_test(async t => {
  const result = await import("./array.json", { with: { type: "json" } });
  assert_array_equals(result.default, ["en", "try"]);
}, "Non-object: array");

