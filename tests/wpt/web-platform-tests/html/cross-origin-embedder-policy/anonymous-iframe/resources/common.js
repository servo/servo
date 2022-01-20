// Create an anonymous iframe. The new document will execute any scripts sent
// toward the token it returns.
const newAnonymousIframe = (child_origin) => {
  const sub_document_token = token();
  let iframe = document.createElement('iframe');
  iframe.src = child_origin + executor_path + `&uuid=${sub_document_token}`;
  iframe.anonymous = true;
  document.body.appendChild(iframe);
  return sub_document_token;
};

// Create a normal iframe. The new document will execute any scripts sent
// toward the token it returns.
const newIframe = (child_origin) => {
  const sub_document_token = token();
  let iframe = document.createElement('iframe');
  iframe.src = child_origin + executor_path + `&uuid=${sub_document_token}`;
  iframe.anonymous = false
  document.body.appendChild(iframe);
  return sub_document_token;
};
