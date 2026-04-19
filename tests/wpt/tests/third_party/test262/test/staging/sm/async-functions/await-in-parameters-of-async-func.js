/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  error for incomplete await expr in async function/generator parameter
info: bugzilla.mozilla.org/show_bug.cgi?id=1478910
esid: pending
---*/

test();

function test()
{
  let testAwaitInDefaultExprOfAsyncFunc = (code) => {
    assert.throws(SyntaxError, () => eval(code), "await expression can't be used in parameter");
  };

  let testNoException = (code) => {
    eval(code);
  };

  // https://www.ecma-international.org/ecma-262/9.0/

  // Async Generator Function Definitions : AsyncGeneratorDeclaration & AsyncGeneratorExpression
  // async function* f() {}
  // f = async function*() {}
  testAwaitInDefaultExprOfAsyncFunc("async function* f(a = await) {}");
  testAwaitInDefaultExprOfAsyncFunc("let f = async function*(a = await) {}");

  testAwaitInDefaultExprOfAsyncFunc("function f(a = async function*(a = await) {}) {}");
  testAwaitInDefaultExprOfAsyncFunc("function f() { a = async function*(a = await) {}; }");

  testAwaitInDefaultExprOfAsyncFunc("async function* f() { a = async function*(a = await) {}; }");
  testNoException("async function* f() { let a = function(a = await) {}; }");

  testNoException("async function* f(a = async function*() { await 1; }) {}");

  // Async Function Definitions : AsyncFunctionDeclaration & AsyncFunctionExpression
  // async function f() {}
  // f = async function() {}
  testAwaitInDefaultExprOfAsyncFunc("async function f(a = await) {}");
  testAwaitInDefaultExprOfAsyncFunc("let f = async function(a = await) {}");

  testAwaitInDefaultExprOfAsyncFunc("function f(a = async function(a = await) {}) {}");
  testAwaitInDefaultExprOfAsyncFunc("function f() { a = async function(a = await) {}; }");

  testAwaitInDefaultExprOfAsyncFunc("async function f() { a = async function(a = await) {}; }");
  testNoException("async function f() { let a = function(a = await) {}; }");

  testNoException("async function f(a = async function() { await 1; }) {}");

  // Async Arrow Function Definitions : AsyncArrowFunction
  // async () => {}
  testAwaitInDefaultExprOfAsyncFunc("async (a = await) => {}");

  testNoException("async (a = async () => { await 1; }) => {}");
}
