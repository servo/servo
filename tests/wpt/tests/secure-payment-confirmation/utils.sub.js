const PAYMENT_DETAILS = {
  total: {label: 'Total', amount: {value: '0.01', currency: 'USD'}}
};
const AUTHENTICATOR_OPTS = {
  protocol: 'ctap2_1',
  transport: 'internal',
  hasResidentKey: true,
  hasUserVerification: true,
  isUserVerified: true,
};

const ICON_URL = 'https://{{hosts[][www]}}:{{ports[https][0]}}/secure-payment-confirmation/troy.png';
const NONEXISTENT_ICON_URL = 'https://{{hosts[][www]}}:{{ports[https][0]}}/secure-payment-confirmation/nonexistent.png';

const ICON_DATA_URL = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAUCAYAAADskT9PAAAAAXNSR0IArs4c6QAAA+lJREFUSEvtlmtQVGUYx3/n7I1dLgtSsIm4ss1EogM4Uanp6IeGqQ81Do0xECyWxKTkWAxDOkmCDYSOmUZjph8wyj7gTDT1oYnJC3ITYrSLkgMul1lCLsouugvs7tnd5pwJx6ZZ40NDfeg5H95z5rzzPv/n8v+/j2C1Wsubmpr26nQ6FQtoHo/Hn5mZWSmYTCbJ6XSqZmdnF9A96PV6jEajXzCbzcGhoaEFdT7nzGw28z+AkBmIioyguLCAXtsAX37zLcFg8B8vU8gSqFQiqSnLOXGkhkH7b+QV7USSJDRaDT6fhCAIqESRWY8HjSigFUU8gQAiAt5AAK1Wg0qlwu/34/X6lP06nVZZJZ+ET5KUYEICyFiVSumOIh6xWLjjcnHmQhtnW9rJ2fQ8Z1vbUKnUrExJpurAEfIS4ticEM9hm52Hw8M4OTRCbl4O6SnJ/HT1GsfrvyBx8UO8sa0Qg17HZ6e/or2z+/4ArNkvUJifTWR4BMNjo3R2/4hjcootuVncdk0jCkE6f+7hZt2nHEg2MyKIvHWlj/oUC7X2US6lpvP6ViujYzfJshbxZvGrvPjcMwwOj7J1Rym3Xa77AzAY9ByqKicjLZWyvdW0dHSxv3I3G9Y8wa+9NuobGhnp7+e75cswCAKbu69w3TXN92tXYdKoyR6+xa6DVQR9EiXl71K9p4xwg5491e9zrqX9bi+FLIFOp+N03VEWm+IpKC6hzzbAqU8+xGJOpKyyhjPNrcRpVfQ/u4HxGQ8Z57sQgMbVaayN1LP+hx62VVWSnLSU1o4uNq5bw8Xuy5S8vQ+P1/v3AIxRETTUHSM22khzRxefNzTy0f59+P0BNuUXMulwKo33cfqjFMQt4uWrNp56IJqC+BguuGbJar9Mbn4Or215iWAgiGPKyc5dFfT0Xv8Tk0JmQKNRU2TN5cnH0hHVao6fPMX2V/IZHZ+g9J0qhRGypRkjKU1K4OsJB9stS7jlclM7PEbzhIP1qx/n8HsVSmbOtV7kYO0xboxPzA+ATJfIiHCijVEIgojD6SR2UQwej1eJXvJLzMmCUaPGHwhiCtPi8PqwrFzBssQE0lNX8PTGdQwM2fng6AkmHVNc67PND4AoiiQtXYLBYFC4OzMzo/Da5Z5WgEAQr1dSKGqMivrj0CBut5uait2YHowlTKfD4bxDxf5DXPqlR9nj8/nmB0DepdFoFLGRHcvCISuhnE75W36XH7knRFFQQMqm1WgpLrQSYTBgH7nB+bYO+gftf3E878tIPlhxfM96bwhz8jwHQP4XE21UgHt9PtzT00iSP6SE/zduQ3kgmZqaUsl1Xki7O5D82yPZ7y210ZoMhOgBAAAAAElFTkSuQmCC';
const INVALID_ICON_DATA_URL = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAUCAYAAADskT9PAAAAAXNSR0IArs4c6QAAA+lJREFUSEvtlmtQVGUYx3/n7I1dLgtSsIm4ss1EogM4Uanp6IeGqQ81Do0xECyWxKTkWAxDOkmCDYSOmUZjph8wyj7gTDT1oYnJC3ITYrSLkgMul1lCLsouugvs7tnd5pwJx6ZZ40NDfeg5H95z5rzzPv/n8v+/j2C1Wsubmpr26nQ6FQtoHo/Hn5mZWSmYTCbJ6XSqZmdnF9A96PV6jEajXzCbzcGhoaEFdT7nzGw28z+AkBmIioyguLCAXtsAX37zLcFg8B8vU8gSqFQiqSnLOXGkhkH7b+QV7USSJDRaDT6fhCAIqESRWY8HjSigFUU8gQAiAt5AAK1Wg0qlwu/34/X6lP06nVZZJZ+ET5KUYEICyFiVSumOIh6xWLjjcnHmQhtnW9rJ2fQ8Z1vbUKnUrExJpurAEfIS4ticEM9hm52Hw8M4OTRCbl4O6SnJ/HT1GsfrvyBx8UO8sa0Qg17HZ6e/or2z+/4ArNkvUJifTWR4BMNjo3R2/4hjcootuVncdk0jCkE6f+7hZt2nHEg2MyKIvHWlj/oUC7X2US6lpvP6ViujYzfJshbxZvGrvPjcThisIsNonsenseG9Y8wa+9NuobGhnp7+e75cswCAKbu69w3TXN92tXYdKoyR6+xa6DVQR9EiXl71K9p4xwg5491e9zrqX9bi+FLIFOp+N03VEWm+IpKC6hzzbAqU8+xGJOpKyyhjPNrcRpVfQ/u4HxGQ8Z57sQgMbVaayN1LP+hx62VVWSnLSU1o4uNq5bw8Xuy5S8vQ+P1/v3AIxRETTUHSM22khzRxefNzTy0f59+P0BNuUXMulwKo33cfqjFMQt4uWrNp56IJqC+BguuGbJar9Mbn4Or215iWAgiGPKyc5dFfT0Xv8Tk0JmQKNRU2TN5cnH0hHVao6fPMX2V/IZHZ+g9J0qhRGypRkjKU1K4OsJB9stS7jlclM7PEbzhIP1qx/n8HsVSmbOtV7kYO0xboxPzA+ATJfIiHCijVEIgojD6SR2UQwej1eJXvJLzMmCUaPGHwhiCtPi8PqwrFzBssQE0lNX8PTGdQwM2fng6AkmHVNc67PND4AoiiQtXYLBYFC4OzMzo/Da5Z5WgEAQr1dSKGqMivrj0CBut5uait2YHowlTKfD4bxDxf5DXPqlR9nj8/nmB0DepdFoFLGRHcvCISuhnE75W36XH7knRFFQQMqm1WgpLrQSYTBgH7nB+bYO+gftf3E878tIPlhxfM96bwhz8jwHQP4XE21UgHt9PtzT00iSP6SE/zduQ3kgmZqaUsl1Xki7O5D82yPZ7y210ZoMhOgBAAAAAElFTkSuQmCC';

// Creates and returns a WebAuthn credential, optionally with the payment
// extension set.
//
// Assumes that a virtual authenticator has already been created.
async function createCredential(set_payment_extension=true) {
  const challengeBytes = new Uint8Array(16);
  window.crypto.getRandomValues(challengeBytes);

  const publicKey = {
    challenge: challengeBytes,
    rp: {
      name: 'Acme',
    },
    user: {
      id: new Uint8Array(16),
      name: 'jane.doe@example.com',
      displayName: 'Jane Doe',
    },
    pubKeyCredParams: [{
      type: 'public-key',
      alg: -7,  // 'ES256'
    }],
    authenticatorSelection: {
      userVerification: 'required',
      residentKey: 'required',
      authenticatorAttachment: 'platform',
    },
    timeout: 30000,
  };

  if (set_payment_extension) {
    publicKey.extensions = {
      payment: { isPayment: true },
    };
  }

  return navigator.credentials.create({publicKey});
}

// Creates a SPC credential in an iframe for the WPT 'alt' domain. Returns a
// promise that resolves with the created credential id.
//
// Assumes that a virtual authenticator has already been created.
async function createCredentialForAltDomain() {
  const frame = document.createElement('iframe');
  frame.allow = 'payment';
  frame.src = 'https://{{hosts[alt][]}}:{{ports[https][0]}}' +
      '/secure-payment-confirmation/resources/iframe-enroll.html';

  // Wait for the iframe to load.
  const readyPromise = new Promise(resolve => {
      window.addEventListener('message', function handler(evt) {
        if (evt.source === frame.contentWindow && evt.data.type == 'loaded') {
          window.removeEventListener('message', handler);

          resolve(evt.data);
        }
      });
  });
  document.body.appendChild(frame);
  await readyPromise;

  // Setup the result promise, and then trigger credential creation.
  const resultPromise = new Promise(resolve => {
      window.addEventListener('message', function handler(evt) {
        if (evt.source === frame.contentWindow && evt.data.type == 'spc_result') {
          document.body.removeChild(frame);
          window.removeEventListener('message', handler);

          resolve(evt.data);
        }
      });
  });
  frame.contentWindow.postMessage({ userActivation: true }, '*');
  return resultPromise;
}

function arrayBufferToString(buffer) {
  return String.fromCharCode(...new Uint8Array(buffer));
}

function base64UrlEncode(data) {
  let result = btoa(data);
  return result.replace(/=+$/g, '').replace(/\+/g, "-").replace(/\//g, "_");
}

