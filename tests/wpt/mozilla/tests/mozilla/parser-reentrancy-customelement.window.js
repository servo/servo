// META: variant=?default
// META: variant=?async

  let script = "\
  class PopupInfo extends HTMLElement { \
    connectedCallback() { \
      frameElement.globalTest.step_timeout(() => frameElement.globalTest.done(), 0); \
      document.open(); \
      document.write('did not panic'); \
      document.close(); \
    } \
  } \
\
customElements.define('popup-info', PopupInfo); \
";

  async_test(function(t) {
    let iframe = document.createElement('iframe');
    iframe.globalTest = t;
    iframe.srcdoc = "<script>" + script + "<" + "/script><popup-info></popup-info>";
    document.body.appendChild(iframe);
  }, "Dynamic markup insertion during custom element callbacks does not panic");
