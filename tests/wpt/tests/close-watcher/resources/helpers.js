window.createRecordingCloseWatcher = (t, events, name, type, parent) => {
  let watcher = null;
  if (type === 'dialog') {
    watcher = document.createElement('dialog');
    watcher.textContent = 'hello world';
    t.add_cleanup(() => watcher.remove());
    if (parent) {
      parent.appendChild(watcher);
    } else {
      document.body.appendChild(watcher);
    }
    watcher.showModal();
  } else if (type === 'popover') {
    watcher = document.createElement('div');
    watcher.setAttribute('popover', 'auto');
    watcher.textContent = 'hello world';
    t.add_cleanup(() => watcher.remove());
    if (parent) {
      parent.appendChild(watcher);
    } else {
      document.body.appendChild(watcher);
    }
    watcher.showPopover();
  } else {
    watcher = new CloseWatcher();
    t.add_cleanup(() => watcher.destroy());
  }

  const prefix = name === undefined ? "" : name + " ";
  watcher.addEventListener('cancel', () => events.push(prefix + "cancel"));
  watcher.addEventListener('close', () => events.push(prefix + "close"));

  return watcher;
};

window.createBlessedRecordingCloseWatcher = async (t, events, name, type, dialog) => {
  return dialogResilientBless(dialog, () => createRecordingCloseWatcher(t, events, name, type, dialog));
};

window.sendEscKey = () => {
  // Esc is \uE00C, *not* \uu001B; see https://w3c.github.io/webdriver/#keyboard-actions.
  //
  // It's important to target document.body, and not any element that might stop receiving events
  // if a popover or dialog is making that element inert.
  return test_driver.send_keys(document.body, '\uE00C');
};

// For now, we always use the Esc keypress as our close request. In
// theory, in the future, we could add a WebDriver command or similar
// for the close request, which would allow different tests on platforms
// with different close requests. In that case, we'd update this
// function, but not update the sendEscKey function above.
window.sendCloseRequest = window.sendEscKey;

// This function is a version of test_driver.bless which works on dialog elements:
// https://github.com/web-platform-tests/wpt/issues/41218
window.dialogResilientBless = async (watcher, fn) => {
  if (watcher instanceof HTMLElement) {
    const button = document.createElement('button');
    watcher.appendChild(button);
    await test_driver.click(button);
    button.remove();
    if (typeof fn === 'function') {
      return fn();
    } else {
      return null;
    }
  } else {
    return await test_driver.bless('dialogResilientBless', fn);
  }
};
