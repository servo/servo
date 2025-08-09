console.log("external module worker");

// Prevent worker exiting before devtools client can query sources
setInterval(() => {}, 0);
