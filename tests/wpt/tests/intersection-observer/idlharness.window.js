// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/IntersectionObserver/

idl_test(
  ['intersection-observer'],
  ['dom'],
  idl_array => {
    idl_array.add_objects({
      IntersectionObserver: ['observer'],
    });
    var options = {
      root: document.body,
      rootMargin: '0px',
      threshold: 1.0
    }
    self.observer = new IntersectionObserver(() => {}, options);
  }
);
