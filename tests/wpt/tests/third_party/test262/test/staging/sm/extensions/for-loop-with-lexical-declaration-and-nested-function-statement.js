/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't assert when freshening the scope chain for a for-loop whose head contains a lexical declaration, where the loop body might add more bindings at runtime
info: bugzilla.mozilla.org/show_bug.cgi?id=1149797
esid: pending
---*/

for (let x = 0; x < 9; ++x) {
  function q1() {}
}

{
  for (let x = 0; x < 9; ++x) {
    function q2() {}
  }
}

function f1()
{
  for (let x = 0; x < 9; ++x) {
    function q3() {}
  }
}
f1();

function f2()
{
  {
    for (let x = 0; x < 9; ++x) {
      function q4() {}
    }
  }
}
f2();

for (let x = 0; x < 9; ++x)
{
  // deliberately inside a block statement
  function q5() {}
}

{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    function q6() {}
  }
}

function g1()
{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    function q7() {}
  }
}
g1();

function g2()
{
  {
    for (let x = 0; x < 9; ++x)
    {
      // deliberately inside a block statement
      function q8() {}
    }
  }
}
g2();

for (let x = 0; x < 9; ++x) {
  (function() {
    eval("function q9() {}");
  })();
}

{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    (function() {
        eval("function q10() {}");
    })();
  }
}

function h1()
{
  for (let x = 0; x < 9; ++x)
  {
    // deliberately inside a block statement
    (function() {
        eval("function q11() {}");
    })();
  }
}
h1();

function h2()
{
  {
    for (let x = 0; x < 9; ++x)
    {
      // deliberately inside a block statement
      (function() { eval("function q12() {}"); })();
    }
  }
}
h2();
