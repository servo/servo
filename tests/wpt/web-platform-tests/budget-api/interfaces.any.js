// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// See https://wicg.github.io/budget-api/

promise_test(async () => {
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const workers = await fetch('/interfaces/dedicated-workers.idl').then(r => r.text());
  const idl = await fetch('/interfaces/budget-api.idl').then(r => r.text());

  const idlArray = new IdlArray();
  idlArray.add_untested_idls(html, { only: [
    'Navigator',
    'NavigatorContentUtils',
    'NavigatorCookies',
    'NavigatorPlugins',
  ] });
  idlArray.add_untested_idls(workers);
  idlArray.add_idls(idl);
  idlArray.test();
  done();
}, 'budget-api interfaces.');
