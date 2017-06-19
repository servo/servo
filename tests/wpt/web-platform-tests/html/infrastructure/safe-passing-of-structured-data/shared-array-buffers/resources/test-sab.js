"use strict";

self.assertSABsHaveSameBackingBlock = (originalSAB, clonedSAB) => {
  const originalView = new Uint8Array(originalSAB);
  const clonedView = new Uint8Array(clonedSAB);

  assert_not_equals(originalSAB, clonedSAB, "the clone must not be the same object");

  assert_equals(originalView[0], 0, "originalView[0] starts 0");
  assert_equals(clonedView[0], 0, "clonedView[0] starts 0");

  originalView[0] = 5;
  assert_equals(originalView[0], 5, "originalView[0] ends up 5");
  assert_equals(clonedView[0], 5, "clonedView[0] ends up 5");
};
