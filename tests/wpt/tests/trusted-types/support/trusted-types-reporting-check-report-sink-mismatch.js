test(() => {
  setTimeout(";"); // violation reported
  setTimeout(";;"); // another violation reported
}, "Passing a plain string to setTimeout works since it's report-only.");

test(() => {
  let p = trustedTypes.createPolicy("dummy", {createScript: x => x});
  setTimeout(p.createScript(";;;")); // no violation reported
}, "Passing a TrustedScript to setTimeout works.");

test(_ => {
  assert_throws_js(EvalError, _ => eval(";;;;")); // violation reported
}, "Passing a plain string to eval throws.");
