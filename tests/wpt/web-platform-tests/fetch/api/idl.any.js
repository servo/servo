// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

promise_test(async() => {
  const text = await (await fetch("/interfaces/fetch.idl")).text();
  const idl_array = new IdlArray();
  idl_array.add_idls(text);
  idl_array.add_untested_idls("[Exposed=(Window,Worker)] interface AbortSignal {};");
  idl_array.add_untested_idls("[Exposed=(Window,Worker)] interface ReadableStream {};");
  idl_array.add_untested_idls("enum ReferrerPolicy {};");
  idl_array.add_objects({
    Headers: ["new Headers()"],
    Request: ["new Request('about:blank')"],
    Response: ["new Response()"],
  });
  idl_array.test();
}, "Fetch Standard IDL");
