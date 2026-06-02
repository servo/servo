// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

"use strict";

// https://wicg.github.io/permissions-policy/

idl_test(
  ['permissions-policy'],
  ['reporting', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Document: ['document'],
      HTMLIFrameElement: ['document.createElement("iframe")'],
      PermissionsPolicy: ['document.permissionsPolicy'],
      // TODO: PermissionsPolicyViolationReportBody
    });
  }
);
