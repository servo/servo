// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://github.com/samuelgoto/sms-receiver

'use strict';

promise_test(async (t) => {
  const srcs = ['./sms_receiver.idl',
                '/interfaces/dom.idl',
                '/interfaces/html.idl'];

  const [sms, dom, html] = await Promise.all(
    srcs.map(i => fetch(i).then(r => r.text()))
  );

  const idl_array = new IdlArray();
  idl_array.add_idls(sms);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  idl_array.add_objects({
    SmsReceiver: ['navigator.sms'],
  });

  idl_array.add_objects({ Navigator: ['navigator'] })

  idl_array.test();
}, 'Test IDL implementation of the SMS Receiver API');

