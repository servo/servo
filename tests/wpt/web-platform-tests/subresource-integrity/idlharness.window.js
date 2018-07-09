// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-subresource-integrity/

'use strict';

promise_test(async () => {
  const srcs = ['webappsec-subresource-integrity', 'html', 'dom', 'cssom'];
  const [idl, html, dom, cssom] = await Promise.all(
      srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(cssom);
  idl_array.add_objects({
    HTMLScriptElement: ['document.createElement("script")'],
    HTMLLinkElement: ['document.createElement("link")'],
  });
  idl_array.test();
}, 'webappsec-subresource-integrity interfaces');
