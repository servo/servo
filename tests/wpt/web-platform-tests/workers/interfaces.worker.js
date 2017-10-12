"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

var request = new XMLHttpRequest();
request.onload = function() {
  var idlArray = new IdlArray();
  var idls = request.responseText;
  idlArray.add_idls(idls);
  idlArray.add_objects({
    DedicatedWorkerGlobalScope: ['self'],
    WorkerNavigator: ['self.navigator'],
    WorkerLocation: ['self.location'],
  });
  idlArray.test();
  done();
};
request.open("GET", "/interfaces/dedicated-workers.idl");
request.send();
