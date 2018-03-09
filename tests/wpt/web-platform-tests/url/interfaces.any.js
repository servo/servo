// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

promise_test(async() => {
  const text = await fetch('/interfaces/url.idl')
    .then(response => response.text());
  const idlArray = new IdlArray();
  idlArray.add_idls(text);
  idlArray.add_objects({
    URL: ['new URL("http://foo")'],
    URLSearchParams: ['new URLSearchParams("hi=there&thank=you")']
  });
  idlArray.test();
  done();
}, 'Test driver');
