// Make sure that the load event for an iframe doesn't fire at the
// point when a navigation triggered by document.write() starts in it,
// but rather when that navigation completes.

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  const doc = frame.contentDocument;
  const url = URL.createObjectURL(new Blob(["PASS"], { type: "text/html"}));

  frame.onload = t.step_func_done(() => {
    assert_equals(frame.contentDocument.body.textContent, "PASS",
                  "Why is our load event firing before the new document loaded?");
  });

  doc.open();
  doc.write(`FAIL<script>location = "${url}"</` + "script>");
  doc.close();
}, "Setting location from document.write() call should not trigger load event until that load completes");

