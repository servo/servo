function createScriptString(origin, referrer) {
  let request_init = referrer ? `{referrer: "${referrer}"}` : "";
  return `<script>
            function checkReferrer() {
              fetch("${origin}/common/security-features/subresource/xhr.py",
                    ${request_init})
                .then(r => r.json())
                .then(j => {
                  top.postMessage({referrer: j.headers.referer}, "*")
                }).catch(e => {
                  top.postMessage({referrer: "FAILURE"}, "*");
                });
            }
            checkReferrer();
            window.addEventListener("message", msg => {
              if (msg.data === "checkReferrer") checkReferrer();
            });
          <\/script>`;
}
