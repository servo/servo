'use strict';

importScripts('/common/dispatcher/dispatcher.js', './executor-common.js');

function addScript(url) {
  importScripts(url);
}

startExecutor();
