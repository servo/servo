"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

async_test(function(t) {
  var request = new XMLHttpRequest();
  request.open("GET", "domstringlist.idl");
  request.send();
  request.onload = t.step_func(function() {
    var idlArray = new IdlArray();
    var idls = request.responseText;

    idlArray.add_idls(idls);

    idlArray.add_objects({
      DOMStringList: [],
    });
    idlArray.test();
    t.done();
  });
});

done();
