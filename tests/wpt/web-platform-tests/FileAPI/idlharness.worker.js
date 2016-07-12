importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

var request = new XMLHttpRequest();
request.open("GET", "idlharness.idl");
request.send();
request.onload = function() {
    var idl_array = new IdlArray();
    var idls = request.responseText;

    idl_array.add_untested_idls("[Global] interface Window { };");

    idl_array.add_untested_idls("interface ArrayBuffer {};");
    idl_array.add_untested_idls("interface ArrayBufferView {};");
    idl_array.add_untested_idls("interface URL {};");
    idl_array.add_untested_idls("interface EventTarget {};");
    idl_array.add_untested_idls("interface Event {};");
    idl_array.add_untested_idls("[TreatNonCallableAsNull] callback EventHandlerNonNull = any (Event event);");
    idl_array.add_untested_idls("typedef EventHandlerNonNull? EventHandler;");

    idl_array.add_idls(idls);

    idl_array.add_objects({
        Blob: ['new Blob(["TEST"])'],
        File: ['new File(["myFileBits"], "myFileName")'],
        FileReader: ['new FileReader()'],
        FileReaderSync: ['new FileReaderSync()']
    });

    idl_array.test();
    done();
};
