// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - module
description: |
  pending
esid: pending
---*/

// Load and instantiate "bug1488117-import-namespace_FIXTURE.js".
// "bug1488117-import-namespace_FIXTURE.js" contains an |import*| request for the current module,
// which triggers GetModuleNamespace for this module. GetModuleNamespace calls GetExportedNames on
// the current module, which in turn resolves and calls GetExportedNames on all |export*| entries.
// And that means HostResolveImportedModule is called for "bug1488117-empty_FIXTURE.js" before
// InnerModuleInstantiation for "bug1488117.js" has resolved that module file.

import "./bug1488117-import-namespace_FIXTURE.js";
export* from "./bug1488117-empty_FIXTURE.js";

