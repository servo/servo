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
/* Direct calls to eval should inherit the strictness of the calling code. */
assert.sameValue(testLenientAndStrict("eval('010')",
                              completesNormally,
                              raisesException(SyntaxError)),
         true);

/*
 * Directives in the eval code itself should always override a direct
 * caller's strictness.
 */
assert.sameValue(testLenientAndStrict("eval('\"use strict\"; 010')",
                              raisesException(SyntaxError),
                              raisesException(SyntaxError)),
         true);

/* Code passed to indirect calls to eval should never be strict. */
assert.sameValue(testLenientAndStrict("var evil=eval; evil('010')",
                              completesNormally,
                              completesNormally),
         true);

/*
 * Code passed to the Function constructor never inherits the caller's
 * strictness.
 */
assert.sameValue(completesNormally("Function('010')"),
         true);
assert.sameValue(raisesException(SyntaxError)("Function('\"use strict\"; 010')"),
         true);

/*
 * If 'eval' causes a frame's primitive |this| to become wrapped, the frame should see the same
 * wrapper object as the eval code.
 */
var call_this, eval_this;
function f(code) {
  /*
   * At this point, a primitive |this| has not yet been wrapped. A
   * reference to |this| from the eval call should wrap it, and the wrapper
   * should be stored where the call frame can see it.
   */
  eval(code);
  call_this = this; 
}
f.call(true, 'eval_this = this');
assert.sameValue(call_this, eval_this);

