
for (const optionParentElementName of ["option", "hr", "select"]) {
  test(() => {
    const select = document.createElement("select");
    const selectedOptions = select.selectedOptions;
    select.innerHTML = "<option>1";

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild);
    assert_equals(select.value, "1");

    const optionParent = select.appendChild(document.createElement(optionParentElementName));

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild);
    assert_equals(select.value, "1");

    const option = optionParent.appendChild(document.createElement("option"));
    option.setAttribute("selected", "");
    option.textContent = "2";

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild);
    assert_equals(select.value, "1");
  }, `<select> containing <${optionParentElementName}><option selected>`);
}

for (const optionParentElementName of ["option", "hr", "select", "optgroup"]) {
  test(() => {
    const select = document.createElement("select");
    const selectedOptions = select.selectedOptions;
    select.innerHTML = "<optgroup><option>1";

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild.firstChild);
    assert_equals(select.value, "1");

    const optionParent = select.firstChild.appendChild(document.createElement(optionParentElementName));

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild.firstChild);
    assert_equals(select.value, "1");

    const option = optionParent.appendChild(document.createElement("option"));
    option.setAttribute("selected", "");
    option.textContent = "2";

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild.firstChild);
    assert_equals(select.value, "1");

  }, `<select><optgroup> containing <${optionParentElementName}><option selected>`);
}

for (const optionParentElementName of ["option", "hr", "select", "optgroup"]) {
  test(() => {
    const select = document.createElement("select");
    select.multiple = true;
    const selectedOptions = select.selectedOptions;
    select.innerHTML = "<div><optgroup><div><option selected>1";

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild.firstChild.firstChild.firstChild);
    assert_equals(select.value, "1");

    const optionParent = select.firstChild.firstChild.firstChild.appendChild(document.createElement(optionParentElementName));

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild.firstChild.firstChild.firstChild);
    assert_equals(select.value, "1");

    const option = optionParent.appendChild(document.createElement("option"));
    option.setAttribute("selected", "");
    option.textContent = "2";

    assert_equals(selectedOptions.length, 1);
    assert_equals(selectedOptions[0], select.firstChild.firstChild.firstChild.firstChild);
    assert_equals(select.value, "1");

    const secondOption = select.appendChild(document.createElement("option"));
    secondOption.setAttribute("selected", "");
    secondOption.textContent = "3";

    assert_equals(selectedOptions.length, 2);
    assert_equals(selectedOptions[0], select.firstChild.firstChild.firstChild.firstChild);
    assert_equals(selectedOptions[1], secondOption);
    assert_equals(select.value, "1");
  }, `<select><div><optgroup><div> containing <${optionParentElementName}><option selected>`);
}
