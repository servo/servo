function createScriptString(origin) {
  return `<script src = "${origin}/common/security-features/resources/common.sub.js"><\/script>
          <script>
            requestViaXhr("${origin}/common/security-features/subresource/xhr.py").then(msg => {
              top.postMessage({referrer: msg.referrer}, "*")
            }).catch(e => {
              top.postMessage({referrer: "FAILURE"}, "*");
            });
          <\/script>`;
}
