let frameCounter = 0;

function populateForm(optionalContentHtml) {
  if (!optionalContentHtml)
    optionalContentHtml = '';
  const frameName = "form-test-target-" + frameCounter++;
  document.body.insertAdjacentHTML(
      'afterbegin',
      `<iframe name="${frameName}"></iframe>` +
          `<form action="/common/blank.html" target="` +
          `${frameName}">${optionalContentHtml}</form>`);
  return document.getElementsByName(frameName)[0].nextSibling;
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
