/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export function attemptGarbageCollection() {
  const w = window;

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
}
//# sourceMappingURL=collect_garbage.js.map