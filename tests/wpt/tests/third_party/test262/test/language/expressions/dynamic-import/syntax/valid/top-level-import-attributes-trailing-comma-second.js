// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-attributes-trailing-comma-second.case
// - src/dynamic-import/syntax/valid/top-level.template
/*---
description: ImportCall trailing comma following second parameter (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [import-attributes, dynamic-import]
flags: [generated]
info: |
    ImportCall :
        import( AssignmentExpression )


    ImportCall :
        import( AssignmentExpression[+In, ?Yield, ?Await] ,opt )
        import( AssignmentExpression[+In, ?Yield, ?Await] , AssignmentExpression[+In, ?Yield, ?Await] ,opt )

---*/

import('./empty_FIXTURE.js', {},);
