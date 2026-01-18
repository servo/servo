// META: title=`Origin.from('file://')`

test(t => {
  const fileURL = "file:///path/to/a/file.txt";
  const opaqueA = Origin.from(fileURL);
  const opaqueB = Origin.from(fileURL);

  assert_true(opaqueA.opaque);
  assert_true(opaqueB.opaque);

  assert_true(opaqueA.isSameOrigin(opaqueA), "Opaque origin should be same-origin with itself.");
  assert_true(opaqueA.isSameSite(opaqueA), "Opaque origin should be same-site with itself.");
  assert_false(opaqueA.isSameOrigin(opaqueB), "Opaque origin should not be same-origin with another opaque origin.");
  assert_false(opaqueA.isSameSite(opaqueB), "Opaque origin should not be same-site with another opaque origin, even if created from same URL.");
}, "`Origin.from('file://')` opaque origins.");
