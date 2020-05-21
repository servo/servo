/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { resolveOnTimeout } from './util.js';
export async function attemptGarbageCollection() {
  const w = self;

  if (w.GCController) {
    w.GCController.collect();
    return;
  }

  if (w.opera && w.opera.collect) {
    w.opera.collect();
    return;
  }

  try {
    w.QueryInterface(Components.interfaces.nsIInterfaceRequestor).getInterface(Components.interfaces.nsIDOMWindowUtils).garbageCollect();
    return;
  } catch (e) {}

  if (w.gc) {
    w.gc();
    return;
  }

  if (w.CollectGarbage) {
    w.CollectGarbage();
    return;
  }

  let i;

  function gcRec(n) {
    if (n < 1) return;
    let temp = {
      i: 'ab' + i + i / 100000
    };
    temp = temp + 'foo';
    gcRec(n - 1);
  }

  for (i = 0; i < 1000; i++) {
    gcRec(10);
  }

  return resolveOnTimeout(35); // Let the event loop run a few frames in case it helps.
}
//# sourceMappingURL=collect_garbage.js.map