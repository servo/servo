setup({ explicit_done: true, explicit_timeout: true });

const applePay = Object.freeze({
  supportedMethods: "https://apple.com/apple-pay",
  data: {
    version: 3,
    merchantIdentifier: "merchant.com.example",
    countryCode: "US",
    merchantCapabilities: ["supports3DS"],
    supportedNetworks: ["visa"],
  }
});

const validMethod = Object.freeze({
  supportedMethods: "basic-card",
});

const validMethods = Object.freeze([validMethod, applePay]);

const validAmount = Object.freeze({
  currency: "USD",
  value: "1.00",
});

const validTotal = Object.freeze({
  label: "Valid total",
  amount: validAmount,
});
const validDetails = {
  total: validTotal,
};

test(() => {
  try {
    new PaymentRequest(validMethods, validDetails);
  } catch (err) {
    done();
    throw err;
  }
}, "Can construct a payment request (smoke test).");

/**
 * Pops up a payment sheet, allowing options to be
 * passed in if particular values are needed.
 *
 * @param PaymentOptions options
 */
async function getPaymentResponse(options, id) {
  const { response } = await getPaymentRequestResponse(options, id);
  return response;
}

/**
 * Creates a payment request and response pair.
 * It also shows the payment sheet.
 *
 * @param {PaymentOptions?} options
 * @param {String?} id
 */
async function getPaymentRequestResponse(options, id) {
  const methods = [{ supportedMethods: "basic-card" }];
  const details = {
    id,
    total: {
      label: "Total due",
      amount: { currency: "USD", value: "1.0" },
    },
  };
  const request = new PaymentRequest(methods, details, options);
  const response = await request.show();
  return { request, response };
}

/**
 * Runs a manual test for payment
 *
 * @param {HTMLButtonElement} button The HTML button.
 * @param {PaymentOptions?} options.
 * @param {Object} expected What property values are expected to pass the test.
 * @param {String?} id And id for the request/response pair.
 */
async function runManualTest(button, options, expected = {}, id = undefined) {
  button.disabled = true;
  const { request, response } = await getPaymentRequestResponse(options, id);
  await response.complete();
  test(() => {
    assert_idl_attribute(
      response,
      "requestId",
      "Expected requestId to be an IDL attribute."
    );
    assert_equals(response.requestId, request.id, `Expected ids to match`);
    for (const [attribute, value] of Object.entries(expected)) {
      assert_idl_attribute(
        response,
        attribute,
        `Expected ${attribute} to be an IDL attribute.`
      );
      assert_equals(
        response[attribute],
        value,
        `Expected response ${attribute} attribute to be ${value}`
      );
    }
    assert_idl_attribute(response, "details");
    assert_equals(typeof response.details, "object", "Expected an object");
    // Testing that this does not throw:
    response.toJSON();
  }, button.textContent.trim());
}
