/**
 * A payment handler that opens a window and allows the user to trigger
 * different types of promise rejections for testing error propagation.
 */

let resolver = null;
let rejecter = null;
let activeMethodName = null;

self.addEventListener('canmakepayment', event => {
  event.respondWith(true);
});

self.addEventListener('message', msgEvent => {
  if (!resolver || !rejecter) return;

  if (msgEvent.data === 'success') {
    resolver({
      methodName: activeMethodName,
      details: {status: 'success'},
    });
  } else if (msgEvent.data === 'reject-operation-error') {
    rejecter(new DOMException('Reject with OperationError', 'OperationError'));
  } else if (msgEvent.data === 'reject-syntax-error') {
    rejecter(new DOMException('Reject with SyntaxError', 'SyntaxError'));
  } else {
    return; // Message not for us.
  }

  resolver = null;
  rejecter = null;
  activeMethodName = null;
});

self.addEventListener('paymentrequest', event => {
  activeMethodName = event.methodData[0].supportedMethods;
  event.respondWith(new Promise((resolve, reject) => {
    resolver = resolve;
    rejecter = reject;

    event.openWindow('payment-app/reject-errors.html').catch(err => {
      resolver = null;
      rejecter = null;
      activeMethodName = null;
      reject(err);
    });
  }));
});
