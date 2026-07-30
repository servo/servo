function populateForm(testId, elementType, formAction, submitterFormAction, formOwnerAction) {
  const frame = document.createElement("iframe");
  const form = document.createElement("form");
  const submitter = document.createElement(elementType);
  let formOwner = null;
  frame.name = `form-test-frame-${elementType}-${testId}`;
  form.id = `form-test-${elementType}-${testId}`;
  submitter.id = `submit-${elementType}-${testId}`;

  if (formOwnerAction !== null) {
    formOwner = document.createElement("form");
    formOwner.target = frame.name;
    formOwner.id = `form-test-${elementType}-${testId-1}`;
    formOwner.setAttribute("action", formOwnerAction);
    submitter.setAttribute(
      "form",
      formOwner.id
    );
    document.body.append(formOwner);
  } else {
    form.target = frame.name;
  }

  if (formAction !== null) {
    form.setAttribute("action", formAction);
  }

  submitter.type = "submit";

  if (submitterFormAction !== null) {
    submitter.setAttribute(
      "formaction",
      submitterFormAction
    );
  }

  form.appendChild(submitter);
  document.body.append(frame, form);
  return frame;
}
