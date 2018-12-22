// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

"use strict";

// https://wicg.github.io/feature-policy/

idl_test(
  ['feature-policy'],
  ['reporting', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Document: ['document'],
      HTMLIframeElement: ['document.createElement("iframe")'],
      FeaturePolicy: ['document.featurePolicy'],
    })
  }
);
