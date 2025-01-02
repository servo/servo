// META: global=window-module
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js
// META: variant=?includeAppServerKey=true
// META: variant=?includeAppServerKey=false

import { encrypt } from "./resources/helpers.js"
import { createVapid } from "./resources/vapid.js";

const includeAppServerKey = new URL(location.href).searchParams.get("includeAppServerKey") === "true";
let registration;

async function subscribe(t) {
  if (includeAppServerKey) {
    const vapid = await createVapid();
    const subscription = await registration.pushManager.subscribe({
      applicationServerKey: vapid.publicKey
    });
    t.add_cleanup(() => subscription.unsubscribe());
    return { vapid, subscription };
  }

  // without key
  try {
    const subscription = await registration.pushManager.subscribe();
    t.add_cleanup(() => subscription.unsubscribe());
    return { subscription };
  } catch (err) {
    if (err.name === "NotSupportedError") {
      // happens if and only if applicationServerKey omission is disallowed,
      // which is permitted per the spec. Throwing OptionalFeatureUnsupportedError marks the
      // result as PRECONDITION_FAILED.
      //
      // https://w3c.github.io/push-api/#subscribe-method
      // If the options argument does not include a non-null value for the applicationServerKey
      // member, and the push service requires one to be given, queue a global task on the
      // networking task source using global to reject promise with a "NotSupportedError"
      // DOMException.
      throw new OptionalFeatureUnsupportedError(description);
    } else {
      throw err;
    }
  }
}

async function pushMessage(subscription, { vapid, message }) {
  const result = !message
    ? { headers: { TTL: 15 } }
    : await encrypt(
      message,
      subscription.getKey("p256dh"),
      subscription.getKey("auth")
    );

  if (includeAppServerKey) {
    result.headers.Authorization = await vapid.generateAuthHeader(
      new URL(subscription.endpoint).origin
    );
  }

  const promise = new Promise(r => {
    navigator.serviceWorker.addEventListener("message", r, { once: true })
  });

  await fetch(subscription.endpoint, {
    method: "post",
    ...result
  });

  return (await promise).data;
}

promise_setup(async () => {
  await trySettingPermission("granted");
  registration = await getActiveServiceWorker("push-sw.js");
});

promise_test(async (t) => {
  const { vapid, subscription } = await subscribe(t);

  const event = await pushMessage(subscription, { vapid });

  assert_equals(event.constructor, "PushEvent");
  assert_equals(event.data, null);
}, "Posting to the push endpoint should fire push event on the service worker");

const entries = [
  { isJSON: false, message: new TextEncoder().encode("Hello") },
  { isJSON: false, message: new Uint8Array([226, 130, 40, 240, 40, 140, 188]) },
  { isJSON: true, message: new TextEncoder().encode(JSON.stringify({ hello: "world" })) },
  { isJSON: false, message: new Uint8Array() },
  { isJSON: false, message: new Uint8Array([0x48, 0x69, 0x21, 0x20, 0xf0, 0x9f, 0x91, 0x80]) },
];

for (const { isJSON, message } of entries) {
  promise_test(async (t) => {
    const { vapid, subscription } = await subscribe(t);

    const event = await pushMessage(subscription, { vapid, message });

    assert_equals(event.constructor, "PushEvent");
    assert_array_equals(new Uint8Array(event.data.arrayBuffer), message);
    assert_array_equals(new Uint8Array(await event.data.blob.arrayBuffer()), message);
    assert_equals(event.data.text, new TextDecoder().decode(message));

    assert_equals(event.data.json.ok, isJSON);
    if (isJSON) {
      assert_array_equals(
        new TextEncoder().encode(JSON.stringify(event.data.json.value)),
        message
      );
    }

  }, `Posting to the push endpoint with encrypted data ${message} should fire push event on the service worker`);
}
