/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Escape sequences aren't allowed in bolded grammar tokens (that is, in keywords, possibly contextual keywords)
info: bugzilla.mozilla.org/show_bug.cgi?id=1204027
esid: pending
---*/

function memberVariants(code)
{
  return ["(class { constructor() {} " + code + " });",
          "({ " + code + " })"];
}

var badScripts =
  [
   "class { st\\u0061tic m() { return 0; } }",
   "class { st\\u0061tic get foo() { return 0; } }",
   "class { st\\u0061tic set foo(v) {} }",
   "class { st\\u0061tic get ['hi']() { return 0; } }",
   "class { st\\u0061tic set ['hi'](v) {} }",
   "class { st\\u0061tic get 'hi'() { return 0; } }",
   "class { st\\u0061tic set 'hi'(v) {} }",
   "class { st\\u0061tic get 42() { return 0; } }",
   "class { st\\u0061tic set 42(v) {} }",
   ...memberVariants("\\u0067et foo() { return 0; }"),
   ...memberVariants("\\u0073et foo() {}"),
   ...memberVariants("g\\u0065t foo() { return 0; }"),
   ...memberVariants("s\\u0065t foo() {}"),
   ...memberVariants("g\\u0065t ['hi']() { return 0; }"),
   ...memberVariants("s\\u0065t ['hi']() {}"),
   ...memberVariants("g\\u0065t 'hi'() { return 0; }"),
   ...memberVariants("s\\u0065t 'hi'() {}"),
   ...memberVariants("g\\u0065t 42() { return 0; }"),
   ...memberVariants("s\\u0065t 42() {}"),
   "for (var foo o\\u0066 [1]) ;",
   "for (var foo \\u006ff [1]) ;",
   "for (var foo i\\u006e [1]) ;",
   "for (var foo \\u0069n [1]) ;",
   "function f() { return n\\u0065w.target }",
   "function f() { return \\u006eew.target }",
   "function f() { return new.t\\u0061rget }",
   "function f() { return new.\\u0074arget }",
   "function f() { return n\\u0065w Array }",
   "function f() { return \\u006eew Array }",
   "\\u0064o {  } while (0)",
   "[for (x \\u006ff [1]) x]",
   "[for (x o\\u0066 [1]) x]",
  ];

for (var script of badScripts)
  assert.throws(SyntaxError, () => Function(script));
