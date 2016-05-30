importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

var request = new XMLHttpRequest();
request.open("GET", "WebCryptoAPI.idl");
request.send();
request.onload = function() {
    var idl_array = new IdlArray();
    var idls = request.responseText;

    idl_array.add_untested_idls("[Global] interface Window { };");

    idl_array.add_untested_idls("interface ArrayBuffer {};");
    idl_array.add_untested_idls("interface ArrayBufferView {};");

    idl_array.add_idls(idls);

    idl_array.add_objects({"Crypto":["crypto"], "SubtleCrypto":["crypto.subtle"]});

    idl_array.test();
    done();
};
