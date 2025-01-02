import { insertCustomIframe, testSupportScript } from "./helpers.mjs";
import { waitForIframe, testGetter } from "../../resources/helpers.mjs";

const testSupportScriptSuitableForNesting =
  testSupportScript.replace('</script>', '</scri` + `pt>');

export default () => {
  promise_setup(async () => {
    const jsURL = `javascript:'${testSupportScript}'`;
    const iframe = await insertCustomIframe(`data:text/html,
      Start page
      <script>
        window.onmessage = () => {
          location.href = \`javascript:'End page${testSupportScriptSuitableForNesting}'\`;
        };
      </script>
    `);

    const waitPromise = waitForIframe(iframe, "javascript: URL");

    // Kick off the navigation. We can't do it directly because only same-origin
    // pages can navigate to a javascript: URL, and we're not same-origin with
    // a data: URL.
    iframe.contentWindow.postMessage(undefined, "*");

    await waitPromise;
  });

  // The javascript: URL iframe inherits its origin from the previous occupant
  // of the iframe, which is a data: URL, so it should always be true.

  testGetter(0, true);
};
