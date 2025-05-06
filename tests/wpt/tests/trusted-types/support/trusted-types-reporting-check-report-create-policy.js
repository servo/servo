test(() => {
  const p1 = trustedTypes.createPolicy("one", {}); // allowed policy name
  assert_equals(p1.name, "one");

  const p2 = trustedTypes.createPolicy("two", {}); // allowed policy name
  assert_equals(p2.name, "two");
}, "Creating policy works for policy in the allowlist.");

test(() => {
  const p3 = trustedTypes.createPolicy("three", {}); // forbidden policy name
  assert_equals(p3.name, "three");

  const p4 = trustedTypes.createPolicy("four", {}); // forbidden policy name
  assert_equals(p4.name, "four");
}, "Creating policy works for policy in the blocklist, since it's report-only.");
