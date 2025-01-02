window.parent.document.test.step_timeout(() => {
  document.write("document.write body contents\n")
  document.close();
  window.parent.document.dispatchEvent(new CustomEvent("documentWriteDone"));
}, 0);
