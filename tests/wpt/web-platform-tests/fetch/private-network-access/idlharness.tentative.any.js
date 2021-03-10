// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://wicg.github.io/private-network-access/

idl_test(
  ['private-network-access'],
  ['html', 'dom'],
  idlArray => {
    if (self.GLOBAL.isWorker()) {
      idlArray.add_objects({
        WorkerGlobalScope: ['self'],
      });
    } else {
      idlArray.add_objects({
        Document: ['document'],
      });
    }
  }
);
