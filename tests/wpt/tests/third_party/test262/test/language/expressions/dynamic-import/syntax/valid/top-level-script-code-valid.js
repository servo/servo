// This file was procedurally generated from the following sources:
// - src/dynamic-import/script-code-valid.case
// - src/dynamic-import/syntax/valid/top-level.template
/*---
description: import() can be used in script code (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [dynamic-import]
flags: [generated]
info: |
    ImportCall :
        import( AssignmentExpression )

---*/
// This is still valid in script code, and should not be valid for module code
// https://tc39.github.io/ecma262/#sec-scripts-static-semantics-lexicallydeclarednames
var smoosh; function smoosh() {}


import('./empty_FIXTURE.js');
