function setupTest() {
  createPassingNotice();
  createTestDiv();
}

function createPassingNotice() {
  const notice = document.createElement('p');

  notice.textContent =
      'Test passes if the image below is green when devicePixelRatio is 1, not red.';

  document.body.appendChild(notice);
}

function createTestDiv() {
  const testDiv = document.createElement('div');

  testDiv.id = 'test';

  testDiv.style.width = '100px';
  testDiv.style.height = '100px';
  testDiv.style.backgroundColor = 'red';

  document.body.appendChild(testDiv);
}

document.addEventListener("DOMContentLoaded", setupTest);
