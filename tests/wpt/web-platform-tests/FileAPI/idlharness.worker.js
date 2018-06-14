importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

promise_test(async () => {
  const idl = await fetch('/interfaces/FileAPI.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const url = await fetch('/interfaces/url.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(url);
  idl_array.add_untested_idls("[Exposed=(Window,Worker)] interface ArrayBuffer {};");
  idl_array.add_objects({
    Blob: ['new Blob(["TEST"])'],
    File: ['new File(["myFileBits"], "myFileName")'],
    FileReader: ['new FileReader()'],
    FileReaderSync: ['new FileReaderSync()']
  });

  idl_array.test();
}, 'Test FileAPI IDL implementation');
done();
