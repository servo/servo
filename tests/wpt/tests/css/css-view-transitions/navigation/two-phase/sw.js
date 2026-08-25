// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
const pause_conditions = {};
async function pause(condition) {
  if (condition)
    await new Promise(resolve => {
      pause_conditions[condition] = resolve;
    });
}

function resume(condition) {
  const resolve = pause_conditions[condition];
  if (resolve) {
    resolve();
    delete pause_conditions[condition];
  }
}

onmessage = async (message) => {
  if ('resume' in message.data) {
    resume(message.data.resume);
  }
};

onfetch = async (event) => {
  const response = fetch(event.request);
  if (!event.request.url.includes('pause')) {
    event.respondWith(response);
    return;
  }

  event.respondWith(pause(new URL(event.request.url).searchParams.get('pause'))
                        .then(() => response));
};