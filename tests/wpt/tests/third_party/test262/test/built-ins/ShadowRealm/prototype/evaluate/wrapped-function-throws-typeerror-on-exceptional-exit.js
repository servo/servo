// Copyright (C) 2021 Igalia SL. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrapped-function-exotic-objects-call-thisargument-argumentslist
description: >
  WrappedFunction throws a TypeError if the wrapped function throws.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.sameValue(
  r.evaluate(`(f) => {
    try {
      f();
    } catch (e) {
      if (e instanceof TypeError) {
        return 'ok';
      } else {
        return e.toString();
      }
    }
    return 'normal exit';
  }`)(() => { throw Error("ahh"); }),
  'ok',
  'WrappedFunction throws TypeError (from the calling realm) if the wrapped callable throws any exception'
);

assert.throws(
  TypeError,
  () => r.evaluate('() => { throw new Error("ahh"); }')(),
  'WrappedFunction throws TypeError if the wrapped callable throws any exception'
);
