// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://www.w3.org/TR/geolocation-API/

promise_test(async () => {
  const idl = await fetch('/interfaces/geolocation-API.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_objects({
    Navigator: ["navigator"],
    Geolocation: ["navigator.geolocation"]
  });
  idl_array.test();
}, 'geolocation-API interfaces');
