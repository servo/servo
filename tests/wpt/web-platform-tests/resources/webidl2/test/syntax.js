"use strict";

const wp = require("../lib/webidl2");
const expect = require("expect");
const pth = require("path");
const fs = require("fs");
const jdp = require("jsondiffpatch");
const debug = true;

describe("Parses all of the IDLs to produce the correct ASTs", () => {
  const dir = pth.join(__dirname, "syntax/idl");
  const skip = {}; // use if we have a broken test
  const idls = fs.readdirSync(dir)
    .filter(it => (/\.widl$/).test(it) && !skip[it])
    .map(it => pth.join(dir, it));
  const jsons = idls.map(it => pth.join(__dirname, "syntax/json", pth.basename(it).replace(".widl", ".json")));

  for (let i = 0, n = idls.length; i < n; i++) {
    const idl = idls[i];
    const json = jsons[i];

    it(`should produce the same AST for ${idl}`, () => {
      try {
        const optFile = pth.join(__dirname, "syntax/opt", pth.basename(json));
        let opt = undefined;
        if (fs.existsSync(optFile))
          opt = JSON.parse(fs.readFileSync(optFile, "utf8"));
        const diff = jdp.diff(JSON.parse(fs.readFileSync(json, "utf8")),
          wp.parse(fs.readFileSync(idl, "utf8").replace(/\r\n/g, "\n"), opt));
        if (diff && debug) console.log(JSON.stringify(diff, null, 4));
        expect(diff).toBe(undefined);
      }
      catch (e) {
        console.log(e.toString());
        throw e;
      }
    });
  }
});
