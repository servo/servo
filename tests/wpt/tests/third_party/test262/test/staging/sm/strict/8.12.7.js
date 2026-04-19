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
function setup() {
  var o = {all: 1, nowrite: 1, noconfig: 1, noble: 1};
  Object.defineProperty(o, 'nowrite', {writable: false});
  Object.defineProperty(o, 'noconfig', {configurable: false});
  Object.defineProperty(o, 'noble', {writable: false, configurable: false});
  return o;
}

assert.sameValue(testLenientAndStrict('var o = setup(); delete o.all',
                              returns(true), returns(true)),
         true);

assert.sameValue(testLenientAndStrict('var o = setup(); delete o.nowrite',
                              returns(true), returns(true)),
         true);

assert.sameValue(testLenientAndStrict('var o = setup(); delete o.noconfig',
                              returns(false), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('var o = setup(); delete o.noble',
                              returns(false), raisesException(TypeError)),
         true);

