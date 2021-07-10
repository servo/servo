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

function submitPromise(form, iframe) {
  return new Promise((resolve, reject) => {
    iframe.onload = () => resolve(iframe.contentWindow.location.search);
    iframe.onerror = () => reject(new Error('iframe onerror fired'));
    form.submit();
  });
}

function loadPromise(iframe) {
  return new Promise((resolve, reject) => {
    iframe.onload = resolve;
    iframe.onerror = () => reject(new Error('iframe onerror fired'));
  });
}
