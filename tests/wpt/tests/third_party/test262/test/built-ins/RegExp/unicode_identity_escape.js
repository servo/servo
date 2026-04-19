// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: IdentityEscape for Unicode RegExp
info: |
    IdentityEscape for Unicode RegExps is restricted to SyntaxCharacter and U+002F (SOLIDUS)
es6id: 21.1.2
---*/

// 21.2.1 Patterns
//
// IdentityEscape[U] ::
//   [+U] SyntaxCharacter
//   [+U] /
//
// SyntaxCharacter :: one of
//   ^ $ \ . * + ? ( ) [ ] { } |

// IdentityEscape in AtomEscape
assert(/\^/u.test("^"), "IdentityEscape in AtomEscape: /\\^/");
assert(/\$/u.test("$"), "IdentityEscape in AtomEscape: /\\$/");
assert(/\\/u.test("\\"), "IdentityEscape in AtomEscape: /\\\\/");
assert(/\./u.test("."), "IdentityEscape in AtomEscape: /\\./");
assert(/\*/u.test("*"), "IdentityEscape in AtomEscape: /\\*/");
assert(/\+/u.test("+"), "IdentityEscape in AtomEscape: /\\+/");
assert(/\?/u.test("?"), "IdentityEscape in AtomEscape: /\\?/");
assert(/\(/u.test("("), "IdentityEscape in AtomEscape: /\\(/");
assert(/\)/u.test(")"), "IdentityEscape in AtomEscape: /\\)/");
assert(/\[/u.test("["), "IdentityEscape in AtomEscape: /\\[/");
assert(/\]/u.test("]"), "IdentityEscape in AtomEscape: /\\]/");
assert(/\{/u.test("{"), "IdentityEscape in AtomEscape: /\\{/");
assert(/\}/u.test("}"), "IdentityEscape in AtomEscape: /\\}/");
assert(/\|/u.test("|"), "IdentityEscape in AtomEscape: /\\|/");
assert(/\//u.test("/"), "IdentityEscape in AtomEscape: /\\//");


// IdentityEscape in ClassEscape
assert(/[\^]/u.test("^"), "IdentityEscape in ClassEscape: /[\\^]/");
assert(/[\$]/u.test("$"), "IdentityEscape in ClassEscape: /[\\$]/");
assert(/[\\]/u.test("\\"), "IdentityEscape in ClassEscape: /[\\\\]/");
assert(/[\.]/u.test("."), "IdentityEscape in ClassEscape: /[\\.]/");
assert(/[\*]/u.test("*"), "IdentityEscape in ClassEscape: /[\\*]/");
assert(/[\+]/u.test("+"), "IdentityEscape in ClassEscape: /[\\+]/");
assert(/[\?]/u.test("?"), "IdentityEscape in ClassEscape: /[\\?]/");
assert(/[\(]/u.test("("), "IdentityEscape in ClassEscape: /[\\(]/");
assert(/[\)]/u.test(")"), "IdentityEscape in ClassEscape: /[\\)]/");
assert(/[\[]/u.test("["), "IdentityEscape in ClassEscape: /[\\[]/");
assert(/[\]]/u.test("]"), "IdentityEscape in ClassEscape: /[\\]]/");
assert(/[\{]/u.test("{"), "IdentityEscape in ClassEscape: /[\\{]/");
assert(/[\}]/u.test("}"), "IdentityEscape in ClassEscape: /[\\}]/");
assert(/[\|]/u.test("|"), "IdentityEscape in ClassEscape: /[\\|]/");
assert(/[\/]/u.test("/"), "IdentityEscape in ClassEscape: /[\\/]/");
