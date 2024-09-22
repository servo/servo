'use strict';

importScripts('/resources/testharness.js');
importScripts('/common/utils.js');
importScripts('/common/dispatcher/dispatcher.js');

function send_message(message) {
  return new Promise((resolve, reject) => {
    const id = token();
    message.id = id;

    addEventListener('message', function listener(e) {
      if (!e.data.command || e.data.id !== id) {
        return;
      }

      removeEventListener('message', listener);

      if (e.data.command !== message.command) {
        reject(`Expected reply with command '${message.command}', got '${
            e.data.command}' instead`);
        return;
      }
      if (e.data.error) {
        reject(e.data.error);
        return;
      }
      resolve();
    });

    postMessage(message);
  });
}

function create_virtual_pressure_source(source, options = {}) {
  return send_message({command: 'create', params: [source, options]});
}

function remove_virtual_pressure_source(source) {
  return send_message({command: 'remove', params: [source]});
}

function update_virtual_pressure_source(source, state) {
  return send_message({command: 'update', params: [source, state]});
}

const uuid = new URLSearchParams(location.search).get('uuid');
const executor = new Executor(uuid);
