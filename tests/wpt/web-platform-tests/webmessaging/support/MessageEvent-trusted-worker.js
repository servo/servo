"use strict";

const channel = new BroadcastChannel("channel name");
channel.postMessage("ping");
