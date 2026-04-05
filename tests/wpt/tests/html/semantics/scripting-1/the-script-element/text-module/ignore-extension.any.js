// META: global=window,dedicatedworker,sharedworker

for (const name of ["file", "file.js", "file.json", "file.txt"]) {
  promise_test(async t => {
    const result = await import(`./${name}`, { with: { type: "text" } });
    assert_equals(result, "text file\n");
  }, `Extension: ${name}`);
}
