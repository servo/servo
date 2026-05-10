/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't crash compiling a non-body-level for-loop whose loop declaration is a const
info: bugzilla.mozilla.org/show_bug.cgi?id=1146644
esid: pending
---*/

// Don't attempt execution as a script if we can't properly emulate it.  We
// could perhaps use eval, but eval, while also doing global execution, is its
// own can of messiness.  Ongoing work on for-loop scoping for lexical
// declarations will likely make these tests redundant with other tests to be
// added, anyway, in the very short term.
var executeGlobalScript = typeof evaluate === "function"
                        ? evaluate
                        : function(s) {};

for (const a1 = 3; false; )
  continue;

Function(`for (const a2 = 3; false; )
            continue;
         `)();

if (true)
{
  for (const a3 = 3; false; )
    continue;
}

Function(`if (true)
          {
            for (const a4 = 3; false; )
              continue;
          }`)();

executeGlobalScript(`for (const a5 of [])
                       continue;`);

Function(`for (const a6 of [])
            continue;`)();

executeGlobalScript(`if (true)
                     {
                       for (const a7 of [])
                         continue;
                     }`);

Function(`if (true)
          {
            for (const a8 of [])
              continue;
          }`)();

executeGlobalScript(`for (const a9 in {})
                       continue;`);

Function(`for (const a10 in {})
            continue;`)();

executeGlobalScript(`if (true)
                     {
                       for (const a11 in {})
                         continue;
                     }`);

Function(`if (true)
          {
            for (const a12 in {})
              continue;
          }`)();
