// META: global=window-module
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

import { encrypt } from "./resources/helpers.js"
import { createVapid } from "./resources/vapid.js";

let registration;

promise_setup(async () => {
  registration = await getActiveServiceWorker("push-sw.js");
});

promise_test(async (t) => {
  await trySettingPermission("granted");

  const vapid = await createVapid();
  const subscription = await registration.pushManager.subscribe({
    applicationServerKey: vapid.publicKey
  });
  t.add_cleanup(() => subscription.unsubscribe());

  await fetch(subscription.endpoint, {
    method: "post",
    headers: {
      TTL: 15,
      Authorization: await vapid.generateAuthHeader(new URL(subscription.endpoint).origin),
    }
  });

  const { data } = await new Promise(r => navigator.serviceWorker.addEventListener("message", r, { once: true }));
  assert_equals(data.constructor, "PushEvent");
}, "Posting to the push endpoint should fire push event on the service worker");

promise_test(async (t) => {
  await trySettingPermission("granted")

  const vapid = await createVapid();
  const subscription = await registration.pushManager.subscribe({
    applicationServerKey: vapid.publicKey
  });
  t.add_cleanup(() => subscription.unsubscribe());

  const { body, headers } = await encrypt(
    new TextEncoder().encode("Hello"),
    subscription.getKey("p256dh"),
    subscription.getKey("auth")
  );

  await fetch(subscription.endpoint, {
    method: "post",
    body,
    headers: {
      ...headers,
      Authorization: await vapid.generateAuthHeader(new URL(subscription.endpoint).origin),
    }
  });

  const { data } = await new Promise(r => navigator.serviceWorker.addEventListener("message", r, { once: true }));
  assert_equals(data.constructor, "PushEvent");
  assert_equals(new TextDecoder().decode(data.data), "Hello");

}, "Posting to the push endpoint with encrypted data should fire push event on the service worker");
