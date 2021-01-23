// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://immersive-web.github.io/dom-overlays/

idl_test(
  ['dom-overlays'],
  ['webxr', 'html', 'dom', 'SVG'],
  async idl_array => {
    self.svgElement = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    idl_array.add_objects({
      Document: ['document'],
      HTMLElement: ['document.body'],
      SVGElement: ['svgElement'],
      Window: ['window']
    });
  }
);
