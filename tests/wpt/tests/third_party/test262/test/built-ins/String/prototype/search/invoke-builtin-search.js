// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of @@search property of internally-created RegExps
es6id: 21.1.3.15
info: |
    [...]
    6. Let rx be RegExpCreate(regexp, undefined) (see 21.2.3.2.3).
    7. ReturnIfAbrupt(rx).
    8. Return Invoke(rx, @@search, «S»).
features: [Symbol.search]
---*/

var originalSearch = RegExp.prototype[Symbol.search];
var returnVal = {};
var result, thisVal, args;

// Fail early if the method is undefined. This test's cleanup logic would
// otherwise install the value `undefined` to the `Symbol.search` property of
// the built-in prototype.
assert.notSameValue(originalSearch, undefined);

RegExp.prototype[Symbol.search] = function() {
  thisVal = this;
  args = arguments;
  return returnVal;
};

try {
  result = 'target'.search('string source');

  assert(thisVal instanceof RegExp);
  assert.sameValue(thisVal.source, 'string source');
  assert.sameValue(thisVal.flags, '');
  assert.sameValue(args.length, 1);
  assert.sameValue(args[0], 'target');
  assert.sameValue(result, returnVal);
} finally {
  RegExp.prototype[Symbol.search] = originalSearch;
}
