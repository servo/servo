// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: >
    Behavior when pattern is a string describing an invalid pattern
    (non-unicode)
info: |
    [...]
    5. Return ? RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize

    6. If F contains "u", let BMP be false; else let BMP be true.
    7. If BMP is true, then
       [...]
    8. Else,
       a. Parse P using the grammars in 21.2.1 and interpreting P as UTF-16
          encoded Unicode code points (6.1.4). The goal symbol for the parse is
          Pattern[U]. Throw a SyntaxError exception if P did not conform to the
          grammar, if any elements of P were not matched by the parse, or if
          any Early Error conditions exist.
---*/

var subject = /test262/ig;

assert.throws(SyntaxError, function() {
  subject.compile('?');
}, 'invalid pattern: ?');

assert.throws(SyntaxError, function() {
  subject.compile('.{2,1}');
}, 'invalid pattern: .{2,1}');

assert.sameValue(
  subject.toString(),
  new RegExp('test262', 'ig').toString(),
  '[[OriginalSource]] internal slot'
);

assert.sameValue(
  subject.test('TEsT262'), true, '[[RegExpMatcher]] internal slot'
);
