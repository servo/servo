// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/portals/

'use strict';

idl_test(
  ['portals'],
  ['html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      HTMLPortalElement: ['document.createElement("portal")'],
      PortalHost: ['window.portalHost'],
      PortalActivateEvent: ['new PortalActivateEvent("portalactivate")'],
    });
  }
);
