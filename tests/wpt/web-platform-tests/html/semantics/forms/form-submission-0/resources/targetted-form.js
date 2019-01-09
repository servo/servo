let frameCounter = 0;

function populateForm(optionalContentHtml) {
  if (!optionalContentHtml)
    optionalContentHtml = '';
  document.body.insertAdjacentHTML(
      'afterbegin',
      `<iframe name="form-test-target-${frameCounter}"></iframe>` +
          `<form action="/common/blank.html" target="` +
          `form-test-target-${frameCounter}">${optionalContentHtml}</form>`);
  ++frameCounter;
  return document.body.firstChild.nextSibling;
}
