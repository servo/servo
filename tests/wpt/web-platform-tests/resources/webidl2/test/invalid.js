
// NOTES:
//  - the errors actually still need to be reviewed to check that they
//    are fully correct interpretations of the IDLs

var wp = process.env.JSCOV ? require("../lib-cov/webidl2") : require("../lib/webidl2")
,   expect = require("expect")
,   pth = require("path")
,   fs = require("fs")
;
describe("Parses all of the invalid IDLs to check that they blow up correctly", function () {
    var dir = pth.join(__dirname, "invalid/idl")
    ,   skip = {}
    ,   idls = fs.readdirSync(dir)
                  .filter(function (it) { return (/\.w?idl$/).test(it) && !skip[it]; })
                  .map(function (it) { return pth.join(dir, it); })
    ,   errors = idls.map(function (it) { return pth.join(__dirname, "invalid", "json", pth.basename(it).replace(/\.w?idl/, ".json")); })
    ;

    for (var i = 0, n = idls.length; i < n; i++) {
        var idl = idls[i], error = JSON.parse(fs.readFileSync(errors[i], "utf8"));
        var func = (function (idl, err) {
            return function () {
                var error;
                try {
                    var ast = wp.parse(fs.readFileSync(idl, "utf8"));
                    console.log(JSON.stringify(ast, null, 4));
                }
                catch (e) {
                    error = e;
                }
                finally {
                    expect(error).toExist();
                    expect(error.message).toEqual(err.message);
                    expect(error.line).toEqual(err.line);
                }

            };
        }(idl, error));
        it("should produce the right error for " + idl, func);
    }
});
