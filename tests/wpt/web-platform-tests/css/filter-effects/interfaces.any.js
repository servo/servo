// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://drafts.fxtf.org/filter-effects/

promise_test(async () => {
  const filterEffectsIdl = await fetch('/interfaces/filter-effects.idl').then(r => r.text());
  const idlArray = new IdlArray();
  idlArray.add_idls(filterEffectsIdl);
  idlArray.add_untested_idls('interface SVGElement {};');
  idlArray.add_untested_idls('interface SVGURIReference {};');
  idlArray.test();
  done();
}, 'Filter effects interfaces.');
