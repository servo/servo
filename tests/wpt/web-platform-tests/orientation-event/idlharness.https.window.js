// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/deviceorientation/spec-source-orientation.html

'use strict';

promise_test(async () => {
  const idl = await fetch('/interfaces/orientation-event.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());

  var idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);
  idl_array.add_objects({
    Window: ['window'],
    DeviceOrientationEvent: ['new DeviceOrientationEvent("foo")'],
    DeviceMotionEvent: ['new DeviceMotionEvent("foo")'],
  });
  idl_array.test();
}, 'orientation-event interfaces');
