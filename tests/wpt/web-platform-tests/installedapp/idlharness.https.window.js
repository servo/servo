// META: global=window
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/get-installed-related-apps/spec/

const idl = `
dictionary RelatedApplication {
  required USVString platform;
  USVString url;
  DOMString id;
  USVString version;
};

[Exposed=Window]
partial interface Navigator {
  [SecureContext] Promise<sequence<RelatedApplication>> getInstalledRelatedApps();
};`;

test(t => {
  const idl_array = new IdlArray();
  idl_array.add_untested_idls("interface Navigator {};");
  idl_array.add_idls(idl);
  idl_array.add_objects({
    Navigator: ['navigator'],
  });
  idl_array.test();
}, 'IDL test for getInstalledRelatedApps');
