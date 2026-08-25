// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Our __noSuchMethod__ handling should only be called when |this| is an object.

var x = "";
// Reached from interpreter's JSOP_CALLPROP, and js::mjit::ic::CallProp.
try { x.i(); } catch (ex) { }

// Reached from interpreter's JSOP_CALLELEM, and js::mjit::stubs::CallElem.
try { x[x](); } catch (ex) { }

// Reached from js::mjit::stubs::CallProp:
try { true.i(); } catch(ex) { }

