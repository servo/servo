var self = require("sdk/self");
var pageMod = require("sdk/page-mod");

pageMod.PageMod({
  include: "*",
  contentScriptFile: self.data.url('perf.js'),
  attachTo: ["top", "existing"]
});
