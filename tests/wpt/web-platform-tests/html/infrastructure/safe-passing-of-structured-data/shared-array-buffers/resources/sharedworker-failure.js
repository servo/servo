let state = "send-sw-failure"
onconnect = initialE => {
  initialE.source.postMessage(state)
  initialE.source.onmessage = e => {
    if(state === "" && e.data === "send-window-failure") {
      e.postMessage(new SharedArrayBuffer())
    } else {
      e.postMessage("failure")
    }
  }
  initialE.source.onmessageerror = e => {
    if(state === "send-sw-failure") {
      e.postMessage("send-sw-failure-success")
      state = ""
    }
  }
}
