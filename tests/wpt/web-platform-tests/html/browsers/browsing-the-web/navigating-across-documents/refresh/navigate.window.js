async_test(t => {
  const frame = document.createElement("iframe");
  frame.src = "resources/refresh.py"
  frame.onload = t.step_func(() => {
    // Could be better by verifying that resources/refresh.py loads too
    if(frame.contentWindow.location.href === (new URL("resources/refreshed.txt?\u0080\u00FF", self.location)).href) { // Make sure bytes got mapped to code points of the same value
      t.done();
    }
  });
  document.body.appendChild(frame)
}, "When navigating the Refresh header needs to be followed");

async_test(t => {
  const frame = document.createElement("iframe");
  frame.src = "resources/multiple.asis"
  frame.onload = t.step_func(() => {
    // Could be better by verifying that resources/refresh.py loads too
    if(frame.contentWindow.location.href === (new URL("resources/refreshed.txt", self.location)).href) {
      t.done();
    }
  });
  document.body.appendChild(frame)
}, "When there's both a Refresh header and <meta> the Refresh header wins")
