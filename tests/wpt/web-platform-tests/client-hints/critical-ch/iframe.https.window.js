async_test((t) => {
  var iframe = document.createElement("iframe");
  iframe.addEventListener('load', (e) => {
    t.step(()=>{assert_equals(iframe.contentDocument.body.textContent, "FAIL");});
    t.done();
  });
  iframe.src = "resources/echo-critical-hint.py";
  document.body.appendChild(iframe);
}, "Critical-CH iframe");
