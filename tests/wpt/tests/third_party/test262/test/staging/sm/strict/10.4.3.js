/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var obj = {}

function strict() { "use strict"; return this; }
assert.sameValue(strict.call(""), "");
assert.sameValue(strict.call(true), true);
assert.sameValue(strict.call(42), 42);
assert.sameValue(strict.call(null), null);
assert.sameValue(strict.call(undefined), undefined);
assert.sameValue(strict.call(obj), obj);
assert.sameValue(new strict() instanceof Object, true);

/* 
 * The compiler internally converts x['foo'] to x.foo. Writing x[s] where
 * s='foo' is enough to throw it off the scent for now.
 */
var strictString = 'strict';

Boolean.prototype.strict = strict;
assert.sameValue(true.strict(), true);
assert.sameValue(true[strictString](), true);

Number.prototype.strict = strict;
assert.sameValue((42).strict(), 42);
assert.sameValue(42[strictString](), 42);

String.prototype.strict = strict;
assert.sameValue("".strict(), "");
assert.sameValue(""[strictString](), "");

function lenient() { return this; }
assert.sameValue(lenient.call("") instanceof String, true);
assert.sameValue(lenient.call(true) instanceof Boolean, true);
assert.sameValue(lenient.call(42) instanceof Number, true);
assert.sameValue(lenient.call(null), this);
assert.sameValue(lenient.call(undefined), this);
assert.sameValue(lenient.call(obj), obj);
assert.sameValue(new lenient() instanceof Object, true);

var lenientString = 'lenient';

Boolean.prototype.lenient = lenient;
assert.sameValue(true.lenient() instanceof Boolean, true);
assert.sameValue(true[lenientString]() instanceof Boolean, true);

Number.prototype.lenient = lenient;
assert.sameValue(42[lenientString]() instanceof Number, true);

String.prototype.lenient = lenient;
assert.sameValue(""[lenientString]() instanceof String, true);

