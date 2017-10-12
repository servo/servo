// Internal function. Returns [instruction, list] DOM elements.
function setupManualShareTestCommon() {
  const div = document.createElement('div');
  document.body.appendChild(div);

  const instruction = document.createElement('div');
  instruction.id = 'instruction';
  div.appendChild(instruction);

  const shareButton = document.createElement('input');
  shareButton.id = 'share_button';
  shareButton.value = 'Share button';
  shareButton.type = 'button';
  div.appendChild(shareButton);

  let heading = document.createElement('h2');
  heading.innerText = 'Instructions:';
  instruction.appendChild(heading);
  let list = document.createElement('ol');
  instruction.appendChild(list);
  let item = document.createElement('li');
  list.appendChild(item);
  item.innerText = 'Click the Share button.';

  return [instruction, list];
}

// Sets up the page for running manual tests. Automatically creates the
// instructions (based on the parameters) and the share button.
function setupManualShareTest(expected_share_data) {
  const {title, text, url} = expected_share_data;
  let [instruction, list] = setupManualShareTestCommon();
  let item = document.createElement('li');
  list.appendChild(item);
  item.innerText = 'Choose a valid share target.';

  heading = document.createElement('h2');
  heading.innerText = 'Pass the test iff the target app received:';
  instruction.appendChild(heading);

  list = document.createElement('ul');
  instruction.appendChild(list);

  item = document.createElement('li');
  list.appendChild(item);
  item.innerText = `title = "${title}"`;
  item = document.createElement('li');
  list.appendChild(item);
  item.innerText = `text = "${text}"`;
  item = document.createElement('li');
  list.appendChild(item);
  item.innerText = `url = "${url}"`;
}

function setupManualShareTestRequiringCancellation() {
  const [instruction, list] = setupManualShareTestCommon();
  const item = document.createElement('li');
  list.appendChild(item);
  item.innerText = 'Cancel the share dialog.';
}

// Returns a promise. When the user clicks the button, calls
// |click_handler| and resolves the promise with the result.
function callWhenButtonClicked(click_handler) {
  return new Promise((resolve, reject) => {
    document.querySelector('#share_button').onclick = () => {
      try {
        resolve(click_handler());
      } catch (e) {
        reject(e);
      }
    };
  });
}

function getAbsoluteUrl(url) {
  return new URL(url, document.baseURI).toString();
}
