"use strict";
this.addEventListener("message", async (event) => {
  const { method, value } = event.data;
  const postMessageData = { method };
  try {
    await navigator[method](value);
    postMessageData.status = "success";
  } catch (e) {
    postMessageData.status = "error";
    postMessageData.exceptionType = e.name;
    postMessageData.message = e.message;
  } finally {
    event.source.postMessage(postMessageData, "*");
  }
});

const target = this.parent ? this.parent : this;
target.postMessage("ready", "*");
