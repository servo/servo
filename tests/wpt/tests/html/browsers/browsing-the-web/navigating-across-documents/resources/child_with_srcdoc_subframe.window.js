window.onmessage = (e) => {
  assert_true(e.data.includes("load grandchild"));

  let srcdoc_content = `
      <p>Grandchild</p><a id='the_anchor'>The Anchor</a>
      <script>
        document.body.onhashchange = () => {
          window.top.postMessage(location.href, '*');
        };
      </scr` + "ipt>";
  const sandbox = e.data.includes("sandbox");
  if (sandbox) {
    srcdoc_content += `
        <script>
          document.body.onload = () => {
            window.top.postMessage(location.href, '*');
            window.location = 'about:srcdoc#the_anchor';
          }
        </scr` + "ipt>";
  }

  let grandchild_frame = document.createElement('iframe');
  grandchild_frame.onload = () => {
    // Each time the grandchild frame loads, send its location to the
    // parent. If that fails, send the error message.
    // For the sandbox case, the child directly sends the href value before
    // self-navigating.
    if (!sandbox) {
      let result;
      try {
        result = grandchild_frame.contentWindow.location.href;
      } catch (error) {
        result = error;
      }
      e.source.postMessage(result, "*");
    }
  };
  if (sandbox) {
    grandchild_frame.sandbox = "allow-scripts";
  }
  grandchild_frame.srcdoc = srcdoc_content;
  document.body.appendChild(grandchild_frame);
};
