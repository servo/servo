/**
 * TestCase
 *
 * A generic template for test cases
 * Is intended to be overloaded with subclasses that override testObject, testFunction and argOrder
 * The testObject is the default arguments for the testFunction
 * The default testObject can be modified with the modify() method, making it easy to create new tests based on the default
 * The testFunction is the target of the test and is called by the test() method. test() applies the testObject as arguments via toArgs()
 * toArgs() uses argOrder to make sure the resulting array is in the right order of the arguments for the testFunction
 */
class TestCase {
    constructor() {
        this.testFunction = function() {
            throw new Error("Test Function not implemented");
        };
        this.testObject = {};
        this.argOrder = [];
    }

    /**
     * toObject
     *
     * return a copy of the testObject
     */
    toObject() {
        return JSON.parse(JSON.stringify(this.testObject)); // cheap clone
    }

    /**
     * toArgs
     *
     * converts test object to an array that is ordered in the same way as the arguments to the test function
     */
    toArgs() {
        var ret = [];
        // XXX, TODO: this won't necessarily produce the args in the right order
        for (let idx of this.argOrder) {
            ret.push(this.testObject[idx]);
        }
        return ret;
    }

    /**
     * modify
     *
     * update the internal object by a path / value combination
     * e.g. :
     * modify ("foo.bar", 3)
     * accepts three types of args:
     *    "foo.bar", 3
     *    {path: "foo.bar", value: 3}
     *    [{path: "foo.bar", value: 3}, ...]
     */
    modify(arg1, arg2) {
        var mods;

        // check for the two argument scenario
        if (typeof arg1 === "string" && arg2 !== undefined) {
            mods = {
                path: arg1,
                value: arg2
            };
        } else {
            mods = arg1;
        }

        // accept a single modification object instead of an array
        if (!Array.isArray(mods) && typeof mods === "object") {
            mods = [mods];
        }

        // iterate through each of the desired modifications, and call recursiveSetObject on them
        var obj = this.testObject;
        for (let idx in mods) {
            var mod = mods[idx];
            let paths = mod.path.split(".");
            recursiveSetObject(this.testObject, paths, mod.value);
        }

        // iterates through nested `obj` using the `pathArray`, creating the path if it doesn't exist
        // when the final leaf of the path is found, it is assigned the specified value
        function recursiveSetObject(obj, pathArray, value) {
            var currPath = pathArray.shift();
            if (typeof obj[currPath] !== "object") {
                obj[currPath] = {};
            }
            if (pathArray.length > 0) {
                return recursiveSetObject(obj[currPath], pathArray, value);
            }
            obj[currPath] = value;
        }

        return this;
    }

    /**
     * test
     *
     * run the test function with the top-level properties of the test object applied as arguments
     */
    test() {
        return this.testFunction(...this.toArgs());
    }

    /**
     * testArgs
     *
     * calls test() with testObject() and expects it to fail with a TypeError()
     */
    testBadArgs(testDesc) {
        promise_test(function(t) {
            return promise_rejects(t, new TypeError(), this.test());
        }.bind(this), testDesc);
    }
}

/**
 * MakeCredentialTest
 *
 * tests the WebAuthn makeCredential() interface
 */
class MakeCredentialTest extends TestCase {
    constructor() {
        // initialize the parent class
        super();

        // the function to be tested
        this.testFunction = navigator.authentication.makeCredential;

        // the default object to pass to makeCredential, to be modified with modify() for various tests
        // var challenge = Uint8Array.from("Y2xpbWIgYSBtb3VudGFpbg");
        this.testObject = {
            accountInformation: {
                rpDisplayName: "ACME",
                displayName: "John P. Smith",
                name: "johnpsmith@example.com",
                id: "1098237235409872",
                imageUri: "https://pics.acme.com/00/p/aBjjjpqPb.png"
            },
            cryptoParameters: [{
                type: "ScopedCred",
                algorithm: "RSASSA-PKCS1-v1_5",
            }],
            attestationChallenge: Uint8Array.from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]).buffer
        };

        // how to order the properties of testObject when passing them to makeCredential
        this.argOrder = [
            "accountInformation",
            "cryptoParameters",
            "attestationChallenge",
            "options"
        ];

        // enable the constructor to modify the default testObject
        // would prefer to do this in the super class, but have to call super() before using `this.*`
        if (arguments.length) this.modify(...arguments);
    }
}

//************* BEGIN DELETE AFTER 1/1/2017 *************** //
// XXX for development mode only!!
// debug() for debugging purposes... we can drop this later if it is considered ugly
// note that debug is currently an empty function (i.e. - prints no output)
// and debug only prints output if the polyfill is loaded
var debug = function() {};
// if the WebAuthn API doesn't exist load a polyfill for testing
// note that the polyfill only gets loaded if navigator.authentication doesn't exist
// AND if the polyfill script is found at the right path (i.e. - the polyfill is opt-in)
function ensureInterface() {
    return new Promise(function(resolve, reject) {
        if (typeof navigator.authentication !== "object") {
            debug = console.log;

            // dynamic loading of polyfill script by creating new <script> tag and seeing the src=
            var scriptElem = document.createElement("script");
            if (typeof scriptElem !== "object") {
                debug("ensureInterface: Error creating script element while attempting loading polyfill");
                return reject(new Error("ensureInterface: Error creating script element while loading polyfill"));
            }
            scriptElem.type = "application/javascript";
            scriptElem.onload = function() {
                debug("!!! XXX - LOADING POLYFILL FOR WEBAUTHN TESTING - XXX !!!");
                return resolve();
            };
            scriptElem.onerror = function() {
                return reject(new Error("navigator.authentication does not exist"));
            };
            scriptElem.src = "/webauthn/webauthn-polyfill/webauthn-polyfill.js";
            if (document.body) {
                document.body.appendChild(scriptElem);
            } else {
                debug("ensureInterface: DOM has no body");
                return reject(new Error("ensureInterface: DOM has no body"));
            }
        }
    });
}

function standardSetup(cb) {
    return ensureInterface()
        .then(() => {
            if (cb) return cb();
        })
        .catch((err) => {
            return (err);
        });
}
//************* END DELETE AFTER 1/1/2017 *************** //