/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Calls to eval with same code + varying strict mode of script containing eval == fail
info: bugzilla.mozilla.org/show_bug.cgi?id=620130
esid: pending
---*/

function t(code) { return eval(code); }

assert.sameValue(t("'use strict'; try { eval('with (5) 17'); } catch (e) { 'threw'; }"),
         "threw");
assert.sameValue(t("try { eval('with (5) 17'); } catch (e) { 'threw'; }"),
         17);
assert.sameValue(t("'use strict'; try { eval('with (5) 17'); } catch (e) { 'threw'; }"),
         "threw");
