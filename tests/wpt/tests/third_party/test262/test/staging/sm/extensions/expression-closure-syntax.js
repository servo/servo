/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Expression closure syntax is only permitted for functions that constitute entire AssignmentExpressions, not PrimaryExpressions that are themselves components of larger binary expressions
info: bugzilla.mozilla.org/show_bug.cgi?id=1416337
esid: pending
---*/

{
  function assertThrowsSyntaxError(code)
  {
    function testOne(replacement)
    {
      assert.throws(SyntaxError, function() {
        eval(code.replace("@@@", replacement));
      });
    }

    testOne("function");
    testOne("async function");
  }

  assertThrowsSyntaxError("x = ++@@@() 1");
  assertThrowsSyntaxError("x = delete @@@() 1");
  assertThrowsSyntaxError("x = new @@@() 1");
  assertThrowsSyntaxError("x = void @@@() 1");
  assertThrowsSyntaxError("x = +@@@() 1");
  assertThrowsSyntaxError("x = 1 + @@@() 1");
  assertThrowsSyntaxError("x = null != @@@() 1");
  assertThrowsSyntaxError("x = null != @@@() 0 ? 1 : a => {}");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {} !== null");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {}.toString");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {}['toString']");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {}``");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {}()");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {}++");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {} || 0");
  assertThrowsSyntaxError("x = 0 || @@@() 0 ? 1 : a => {}");
  assertThrowsSyntaxError("x = @@@() 0 ? 1 : a => {} && true");
  assertThrowsSyntaxError("x = true && @@@() 0 ? 1 : a => {}");
}
