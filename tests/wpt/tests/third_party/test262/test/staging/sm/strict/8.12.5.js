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
function obj() {
  var o = {all: 1, nowrite: 1, noconfig: 1, noble: 1};
  Object.defineProperty(o, 'nowrite', {writable: false});
  Object.defineProperty(o, 'noconfig', {configurable: false});
  Object.defineProperty(o, 'noble', {writable: false, configurable: false});
  return o;
}

assert.sameValue(testLenientAndStrict('var o = obj(); o.all = 2; o.all',
                              returns(2), returns(2)),
         true);

assert.sameValue(testLenientAndStrict('var o = obj(); o.nowrite = 2; o.nowrite',
                              returns(1), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('var o = obj(); o.noconfig = 2; o.noconfig',
                              returns(2), returns(2)),
         true);

assert.sameValue(testLenientAndStrict('var o = obj(); o.noble = 2; o.noble',
                              returns(1), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('obj().nowrite++',
                              returns(1), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('++obj().nowrite',
                              returns(2), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('obj().nowrite--',
                              returns(1), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('--obj().nowrite',
                              returns(0), raisesException(TypeError)),
         true);


function arr() {
  return Object.defineProperties([1, 1, 1, 1],
                                 { 1: { writable: false },
                                   2: { configurable: false },
                                   3: { writable: false, configurable: false }});
}

assert.sameValue(testLenientAndStrict('var a = arr(); a[0] = 2; a[0]',
                              returns(2), returns(2)),
         true);

assert.sameValue(testLenientAndStrict('var a = arr(); a[1] = 2; a[1]',
                              returns(1), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('var a = arr(); a[2] = 2; a[2]',
                              returns(2), returns(2)),
         true);

assert.sameValue(testLenientAndStrict('var a = arr(); a[3] = 2; a[3]',
                              returns(1), raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('arr()[1]++',
                              returns(1), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('++arr()[1]',
                              returns(2), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('arr()[1]--',
                              returns(1), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('--arr()[1]',
                              returns(0), raisesException(TypeError)),
         true);

