// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of @@match property of internally-created RegExps
es6id: 21.1.3.11
info: |
    [...]
    6. Let rx be RegExpCreate(regexp, undefined) (see 21.2.3.2.3).
    7. ReturnIfAbrupt(rx).
    8. Return Invoke(rx, @@match, «S»).
features: [Symbol.match]
---*/

var originalMatch = RegExp.prototype[Symbol.match];
var returnVal = {};
var result, thisVal, args;

RegExp.prototype[Symbol.match] = function() {
  thisVal = this;
  args = arguments;
  return returnVal;
};

try {
  result = 'target'.match('string source');

  assert(thisVal instanceof RegExp);
  assert.sameValue(thisVal.source, 'string source');
  assert.sameValue(thisVal.flags, '');
  assert.sameValue(thisVal.lastIndex, 0);
  assert.sameValue(args.length, 1);
  assert.sameValue(args[0], 'target');
  assert.sameValue(result, returnVal);
} finally {
  RegExp.prototype[Symbol.match] = originalMatch;
}
