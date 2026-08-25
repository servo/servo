/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-strict-shell.js]
description: |
  pending
esid: pending
---*/
function str() {
  return new String("foo");
}

assert.sameValue(testLenientAndStrict('var s = str(); s.length = 1; s.length',
                              returns(3), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('var s = str(); delete s.length',
                              returns(false), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('"foo".length = 1',
                              returns(1), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('delete "foo".length',
                              returns(false), raisesException(TypeError)),
         true);

