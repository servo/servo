promise_test(async test => {
  const message = new Promise(r => window.addEventListener("message", r));

  const iframe_unsandboxed = document.createElement("iframe");
  document.body.appendChild(iframe_unsandboxed);

  const iframe_sandboxed = document.createElement("iframe");
  iframe_sandboxed.sandbox = "allow-same-origin allow-scripts";
  document.body.appendChild(iframe_sandboxed);

  iframe_sandboxed.srcdoc = `
    <script>
      parent.frames[0].document.write(\`
        <script>
          // Return whether the current context is sandboxed or not. The implementation does
          // not matter much, but might have to change over time depending on what side
          // effect sandbox flags have. Feel free to update as needed.
          try {
            document.domain = document.domain;
            window.parent.postMessage("not sandboxed", "*");
          } catch (error) {
            window.parent.postMessage("sandboxed", "*");
          }
        </scr\`+\`ipt>
      \`);
      parent.frames[0].document.close();
    </scr`+`ipt>
  `;
  assert_equals((await message).data, "not sandboxed");

}, "Using document.open() against a document from a different window must not" +
   " mutate the other window's sandbox flags");
