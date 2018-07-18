// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

async_test(function() {
  const srcs = ['requestidlecallback', 'html', 'dom'];
  Promise.all(srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())))
    .then(([idl, html, dom]) => {
      var idl_array = new IdlArray();
      idl_array.add_idls(idl);
      idl_array.add_dependency_idls(html);
      idl_array.add_dependency_idls(dom);
      idl_array.add_objects({Window: ['window']});

      let deadline;
      const execIDLTest = this.step_func_done(function() {
        idl_array.add_objects({IdleDeadline: [deadline]});
        idl_array.test();
      });

      if (!window.requestIdleCallback) {
        execIDLTest();
      } else {
        const callback = this.step_func(d => {
          deadline = d;
          execIDLTest();
        });
        requestIdleCallback(callback, { timeout: 100 });
      }
    });
}, 'IdleDeadline object setup');
