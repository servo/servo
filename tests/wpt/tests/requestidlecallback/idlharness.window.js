// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/requestidlecallback/

'use strict';

idl_test(
  ['requestidlecallback'],
  ['html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      IdleDeadline: ['deadline'],
      Window: ['window'],
    });

    await new Promise(resolve => {
      requestIdleCallback(d => {
        self.deadline = d;
        resolve();
      }, { timeout: 100 });
    });
  }
);
