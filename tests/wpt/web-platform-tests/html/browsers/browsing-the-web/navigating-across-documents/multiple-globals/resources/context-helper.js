// Usage: in the top-level Window, include:
//
// <script src="resources/context-helper.js"></script>
// <script>
//   window.scriptToRun = '...';
// </script>
// <iframe id="entry" src="entry/entry.html"></iframe>
//
// Then `scriptToRun` is evaluated, with:
// - The entry Realm is that of entry/entry.html
// - The incumbent Realm is that of incumbent/empty.html
// - The relevant Realm of `relevantWindow`, `relevantWindow.location` etc. is
//   that of relevant/empty.html

window.scriptToRun = '';

const entryUrl = new URL('entry/entry.html', location).href;
const incumbentUrl = new URL('incumbent/empty.html', location).href;
const relevantUrl = new URL('relevant/empty.html', location).href;

function go() {
    const entry = document.querySelector('#entry');
    const incumbent = entry.contentDocument.querySelector('#incumbent');
    const incumbentScript = incumbent.contentDocument.createElement('script');
    incumbentScript.textContent = `
    function go() {
      const relevantWindow =
          parent.document.querySelector('#relevant').contentWindow;
      ${window.scriptToRun}
    }
    `;
    incumbent.contentDocument.head.appendChild(incumbentScript);
    incumbent.contentWindow.go();
}
