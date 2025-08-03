// META: title=`Origin` comparison

test(t => {
  const opaqueA = new Origin("null");
  const opaqueB = new Origin("null");

  assert_true(opaqueA.isSameOrigin(opaqueA), "Opaque origin should be same-origin with itself.");
  assert_true(opaqueA.isSameSite(opaqueA), "Opaque origin should be same-site with itself.");
  assert_false(opaqueA.isSameOrigin(opaqueB), "Opaque origin should not be same-origin with another opaque origin.");
  assert_false(opaqueA.isSameSite(opaqueB), "Opaque origin should not be same-site with another opaque origin.");
}, "Comparison of opaque origins.");

test(t => {
  const a = new Origin("https://a.example");
  const a_a = new Origin("https://a.a.example");
  const b_a = new Origin("https://b.a.example");
  const b = new Origin("https://b.example");
  const b_b = new Origin("https://b.b.example");

  assert_true(a.isSameOrigin(a), "Origin should be same-origin with itself.");
  assert_false(a.isSameOrigin(a_a), "Origins with different subdomains should not be same-origin.");
  assert_false(a.isSameOrigin(b_a), "Origins with different subdomains should not be same-origin.");
  assert_false(a.isSameOrigin(b), "Origins with different domains should not be same-origin.");
  assert_false(a.isSameOrigin(b_b), "Origins with different domains should not be same-origin.");

  assert_true(a.isSameSite(a), "Origin should be same-site with itself.");
  assert_true(a.isSameSite(a_a), "Origins with same registrable domain should be same-site.");
  assert_true(a.isSameSite(b_a), "Origins with same registrable domain should be same-site.");
  assert_false(a.isSameSite(b), "Origins with different registrable domains should not be same-site.");
  assert_false(a.isSameSite(b_b), "Origins with different registrable domains should not be same-site.");

  assert_true(a_a.isSameSite(a), "Origins with same registrable domain should be same-site.");
  assert_true(a_a.isSameSite(a_a), "Origin should be same-site with itself.");
  assert_true(a_a.isSameSite(b_a), "Origins with same registrable domain should be same-site.");
  assert_false(a_a.isSameSite(b), "Origins with different registrable domains should not be same-site.");
  assert_false(a_a.isSameSite(b_b), "Origins with different registrable domains should not be same-site.");
}, "Comparison of tuple origins.");
