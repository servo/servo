promise_test(async () => {
  // Exceed Gecko's 5,000-character threshold for off-thread compilation.
  const text = "x".repeat(6000);
  const module = await import(`data:text/plain,${text}`, { with: { type: "text" } });

  assert_equals(module.default, text, "The import should preserve all the text");
}, "Importing a large text module");
