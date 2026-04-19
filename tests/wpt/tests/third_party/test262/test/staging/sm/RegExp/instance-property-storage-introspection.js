/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Represent /a/.{lastIndex,global,source,multiline,sticky,ignoreCase} with plain old data properties
info: bugzilla.mozilla.org/show_bug.cgi?id=640072
esid: pending
---*/

function checkDataProperty(obj, p, expect, msg)
{
  var d = Object.getOwnPropertyDescriptor(obj, p);

  assert.sameValue(d.value, expect.value, msg + ": bad value for " + p);
  assert.sameValue(d.writable, expect.writable, msg + ": bad writable for " + p);
  assert.sameValue(d.enumerable, expect.enumerable, msg + ": bad enumerable for " + p);
  assert.sameValue(d.configurable, expect.configurable, msg + ": bad configurable for " + p);

  // Try redefining the property using its initial values: these should all be
  // silent no-ops.
  Object.defineProperty(obj, p, { value: expect.value });
  Object.defineProperty(obj, p, { writable: expect.writable });
  Object.defineProperty(obj, p, { enumerable: expect.enumerable });
  Object.defineProperty(obj, p, { configurable: expect.configurable });

  var d2 = Object.getOwnPropertyDescriptor(obj, p);
  assert.sameValue(d.value, d2.value, msg + ": value changed on redefinition of " + p + "?");
  assert.sameValue(d.writable, d2.writable, msg + ": writable changed on redefinition of " + p + "?");
  assert.sameValue(d.enumerable, d2.enumerable, msg + ": enumerable changed on redefinition of " + p + "?");
  assert.sameValue(d.configurable, d2.configurable, msg + ": configurable changed on redefinition of " + p + "?");
}


// Check a bunch of "empty" regular expressions first.

var choices = [{ msg: "new RegExp()",
                 get: function() { return new RegExp(); } },
               { msg: "/(?:)/",
                 get: Function("return /(?:)/;") }];

function checkRegExp(r, msg, lastIndex)
{
  var expect;

  expect = { value: lastIndex, enumerable: false, configurable: false, writable: true };
  checkDataProperty(r, "lastIndex", expect, msg);
}

checkRegExp(new RegExp(), "new RegExp()", 0);
checkRegExp(/(?:)/, "/(?:)/", 0);
checkRegExp(Function("return /(?:)/;")(), 'Function("return /(?:)/;")()', 0);

for (var i = 0; i < choices.length; i++)
{
  var choice = choices[i];
  var msg = choice.msg;
  var r = choice.get();

  checkRegExp(r, msg, 0);
}

// Now test less generic regular expressions

checkRegExp(/a/gim, "/a/gim", 0);

var r;

do
{
  r = /abcd/mg;
  checkRegExp(r, "/abcd/mg initially", 0);
  r.exec("abcdefg");
  checkRegExp(r, "/abcd/mg step 1", 4);
  r.exec("abcdabcd");
  checkRegExp(r, "/abcd/mg step 2", 8);
  r.exec("abcdabcd");
  checkRegExp(r, "/abcd/mg end", 0);

  r = /cde/ig;
  checkRegExp(r, "/cde/ig initially", 0);
  var obj = r.lastIndex = { valueOf: function() { return 2; } };
  checkRegExp(r, "/cde/ig after lastIndex", obj);
  r.exec("aaacdef");
  checkRegExp(r, "/cde/ig after exec", 6);
  Object.defineProperty(r, "lastIndex", { value: 3 });
  checkRegExp(r, "/cde/ig after define 3", 3);
  Object.defineProperty(r, "lastIndex", { value: obj });
  checkRegExp(r, "/cde/ig after lastIndex", obj);


  // Tricky bits of testing: make sure that redefining lastIndex doesn't change
  // the slot where the lastIndex property is initially stored, even if
  // the redefinition also changes writability.
  r = /a/g;
  checkRegExp(r, "/a/g initially", 0);
  Object.defineProperty(r, "lastIndex", { value: 2 });
  r.exec("aabbbba");
  checkRegExp(r, "/a/g after first exec", 7);
  assert.sameValue(r.lastIndex, 7);
  r.lastIndex = 2;
  checkRegExp(r, "/a/g after assign", 2);
  r.exec("aabbbba");
  assert.sameValue(r.lastIndex, 7); // check in reverse order
  checkRegExp(r, "/a/g after second exec", 7);

  r = /c/g;
  r.lastIndex = 2;
  checkRegExp(r, "/c/g initially", 2);
  Object.defineProperty(r, "lastIndex", { writable: false });
  assert.sameValue(Object.getOwnPropertyDescriptor(r, "lastIndex").writable, false);
  try { r.exec("aabbbba"); } catch (e) { /* swallow error if thrown */ }
  assert.sameValue(Object.getOwnPropertyDescriptor(r, "lastIndex").writable, false);
}
while (Math.random() > 17); // fake loop to discourage RegExp object caching
