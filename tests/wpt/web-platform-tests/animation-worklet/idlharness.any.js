// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://wicg.github.io/animation-worklet/

idl_test(
  ['css-animation-worklet'],
  ['web-animations', 'html', 'cssom', 'dom'],
  idl_array => {
    idl_array.add_objects({
      WorkletAnimation: ['new WorkletAnimation("name")'],
      // TODO: WorkletGroupEffect
    });
  }
);
