
/* Useful constants for working with COSE key objects */
const cose_kty = 1;
const cose_kty_ec2 = 2;
const cose_alg = 3;
const cose_alg_ECDSA_w_SHA256 = -7;
const cose_alg_ECDSA_w_SHA512 = -36;
const cose_crv = -1;
const cose_crv_P256 = 1;
const cose_crv_x = -2;
const cose_crv_y = -3;

/**
 * TestCase
 *
 * A generic template for test cases
 * Is intended to be overloaded with subclasses that override testObject, testFunction and argOrder
 * The testObject is the default arguments for the testFunction
 * The default testObject can be modified with the modify() method, making it easy to create new tests based on the default
 * The testFunction is the target of the test and is called by the doIt() method. doIt() applies the testObject as arguments via toArgs()
 * toArgs() uses argOrder to make sure the resulting array is in the right order of the arguments for the testFunction
 */
class TestCase {
    constructor() {
        this.testFunction = function() {
            throw new Error("Test Function not implemented");
        };
        this.testObject = {};
        this.argOrder = [];
        this.ctx = null;
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
     * actually runs the test function with the supplied arguments
     */
    doIt() {
        if (typeof this.testFunction !== "function") {
            throw new Error("Test function not found");
        }

        return this.testFunction.call(this.ctx, ...this.toArgs());
    }

    /**
     * run the test function with the top-level properties of the test object applied as arguments
     */
    test(desc) {
        promise_test(() => {
            return this.doIt()
                .then((ret) => {
                    // check the result
                    this.validateRet(ret);
                    return ret;
                });
        }, desc);
    }

    /**
     * validates the value returned from the test function
     */
    validateRet() {
        throw new Error("Not implemented");
    }

    /**
     * calls doIt() with testObject() and expects it to fail with a TypeError()
     */
    testBadArgs(testDesc) {
        promise_test(function(t) {
            return promise_rejects(t, new TypeError(), this.doIt(), "Expected bad parameters to fail");
        }.bind(this), testDesc);
    }
}

var createCredentialDefaultArgs = {
    options: {
        publicKey: {
            // Relying Party:
            rp: {
                name: "Acme"
            },

            // User:
            user: {
                id: new Uint8Array(), // Won't survive the copy, must be rebuilt
                name: "john.p.smith@example.com",
                displayName: "John P. Smith",
                icon: "https://pics.acme.com/00/p/aBjjjpqPb.png"
            },

            pubKeyCredParams: [{
                type: "public-key",
                alg: cose_alg_ECDSA_w_SHA256,
            }],

            timeout: 60000, // 1 minute
            excludeCredentials: [] // No excludeList
        }
    }
};

function cloneObject(o) {
    return JSON.parse(JSON.stringify(o));
}

/**
 * CreateCredentialTest
 *
 * tests the WebAuthn navigator.credentials.create() interface
 */
class CreateCredentialsTest extends TestCase {
    constructor() {
        // initialize the parent class
        super();

        // the function to be tested
        this.testFunction = navigator.credentials.create;
        // the context to call the test function with (i.e. - the 'this' object for the function)
        this.ctx = navigator.credentials;

        // the default object to pass to makeCredential, to be modified with modify() for various tests
        let challengeBytes = new Uint8Array(16);
        window.crypto.getRandomValues(challengeBytes);
        this.testObject = cloneObject(createCredentialDefaultArgs);
        // cloneObject can't clone the BufferSource in user.id, so let's recreate it.
        this.testObject.options.publicKey.user.id = new Uint8Array();
        this.testObject.options.publicKey.challenge = challengeBytes;

        // how to order the properties of testObject when passing them to makeCredential
        this.argOrder = [
            "options"
        ];

        // enable the constructor to modify the default testObject
        // would prefer to do this in the super class, but have to call super() before using `this.*`
        if (arguments.length) this.modify(...arguments);
    }

    validateRet(ret) {
        validatePublicKeyCredential(ret);
        validateAuthenticatorAttestationResponse(ret.response);
    }
}

/**
 * GetCredentialsTest
 *
 * tests the WebAuthn navigator.credentials.get() interface
 */
class GetCredentialsTest extends TestCase {
    constructor(...args) {
        // initialize the parent class
        super();

        // the function to be tested
        this.testFunction = navigator.credentials.get;
        // the context to call the test function with (i.e. - the 'this' object for the function)
        this.ctx = navigator.credentials;

        // default arguments
        let challengeBytes = new Uint8Array(16);
        window.crypto.getRandomValues(challengeBytes);
        this.testObject = {
            options: {
                publicKey: {
                    challenge: challengeBytes,
                    // timeout: 60000,
                    // allowCredentials: [newCredential]
                }
            }
        };

        // how to order the properties of testObject when passing them to makeCredential
        this.argOrder = [
            "options"
        ];

        this.credentialPromiseList = [];

        // enable the constructor to modify the default testObject
        // would prefer to do this in the super class, but have to call super() before using `this.*`
        if (arguments.length) {
            if (args.cred instanceof Promise) this.credPromise = args.cred;
            else if (typeof args.cred === "object") this.credPromise = Promise.resolve(args.cred);
            delete args.cred;
            this.modify(...arguments);
        }
    }

    addCredential(arg) {
        // if a Promise was passed in, add it to the list
        if (arg instanceof Promise) {
            this.credentialPromiseList.push(arg);
            return;
        }

        // if a credential object was passed in, convert it to a Promise for consistency
        if (typeof arg === "object") {
            this.credentialPromiseList.push(Promise.resolve(arg));
            return;
        }

        // if a credential wasn't passed in, create one
        let challengeBytes = new Uint8Array(16);
        window.crypto.getRandomValues(challengeBytes);
        var createArgs = cloneObject(createCredentialDefaultArgs);
        createArgs.options.publicKey.challenge = challengeBytes;
        createArgs.options.publicKey.user.id = new Uint8Array();
        var p = navigator.credentials.create(createArgs.options);
        this.credentialPromiseList.push(p);

        return this;
    }

    test() {
        if (!this.credentialPromiseList.length) {
            throw new Error("Attempting list without defining credential to test");
        }

        Promise.all(this.credentialPromiseList)
            .then((credList) => {
                var idList = credList.map((cred) => {
                    return {
                        id: cred.rawId,
                        transports: ["usb", "nfc", "ble"],
                        type: "public-key"
                    };
                });
                this.testObject.options.publicKey.allowCredentials = idList;
                return super.test();
            });
    }

    validateRet(ret) {
        validatePublicKeyCredential (ret);
        validateAuthenticatorAssertionResponse(ret.response);
    }
}

/**
 * runs assertions against a PublicKeyCredential object to ensure it is properly formatted
 */
function validatePublicKeyCredential(cred) {
    // class
    assert_class_string(cred, "PublicKeyCredential", "Expected return to be instance of 'PublicKeyCredential' class");
    // id
    assert_idl_attribute(cred, "id", "should return PublicKeyCredential with id attribute");
    assert_readonly(cred, "id", "should return PublicKeyCredential with readonly id attribute");
    // rawId
    assert_idl_attribute(cred, "rawId", "should return PublicKeyCredential with rawId attribute");
    assert_readonly(cred, "rawId", "should return PublicKeyCredential with readonly rawId attribute");
    // type
    assert_idl_attribute(cred, "type", "should return PublicKeyCredential with type attribute");
    assert_equals(cred.type, "public-key", "should return PublicKeyCredential with type 'public-key'");
}

/**
 * runs assertions against a AuthenticatorAttestationResponse object to ensure it is properly formatted
 */
function validateAuthenticatorAttestationResponse(attr) {
    // class
    assert_class_string(attr, "AuthenticatorAttestationResponse", "Expected credentials.create() to return instance of 'AuthenticatorAttestationResponse' class");
    // clientDataJSON
    assert_idl_attribute(attr, "clientDataJSON", "credentials.create() should return AuthenticatorAttestationResponse with clientDataJSON attribute");
    assert_readonly(attr, "clientDataJSON", "credentials.create() should return AuthenticatorAttestationResponse with readonly clientDataJSON attribute");
    // attestationObject
    assert_idl_attribute(attr, "attestationObject", "credentials.create() should return AuthenticatorAttestationResponse with attestationObject attribute");
    assert_readonly(attr, "attestationObject", "credentials.create() should return AuthenticatorAttestationResponse with readonly attestationObject attribute");
}

/**
 * runs assertions against a AuthenticatorAssertionResponse object to ensure it is properly formatted
 */
function validateAuthenticatorAssertionResponse(assert) {
    // class
    assert_class_string(assert, "AuthenticatorAssertionResponse", "Expected credentials.create() to return instance of 'AuthenticatorAssertionResponse' class");
    // clientDataJSON
    assert_idl_attribute(assert, "clientDataJSON", "credentials.get() should return AuthenticatorAssertionResponse with clientDataJSON attribute");
    assert_readonly(assert, "clientDataJSON", "credentials.get() should return AuthenticatorAssertionResponse with readonly clientDataJSON attribute");
    // signature
    assert_idl_attribute(assert, "signature", "credentials.get() should return AuthenticatorAssertionResponse with signature attribute");
    assert_readonly(assert, "signature", "credentials.get() should return AuthenticatorAssertionResponse with readonly signature attribute");
    // authenticatorData
    assert_idl_attribute(assert, "authenticatorData", "credentials.get() should return AuthenticatorAssertionResponse with authenticatorData attribute");
    assert_readonly(assert, "authenticatorData", "credentials.get() should return AuthenticatorAssertionResponse with readonly authenticatorData attribute");
}

//************* BEGIN DELETE AFTER 1/1/2018 *************** //
// XXX for development mode only!!
// debug() for debugging purposes... we can drop this later if it is considered ugly
// note that debug is currently an empty function (i.e. - prints no output)
// and debug only prints output if the polyfill is loaded
var debug = function() {};
// if the WebAuthn API doesn't exist load a polyfill for testing
// note that the polyfill only gets loaded if navigator.credentials create doesn't exist
// AND if the polyfill script is found at the right path (i.e. - the polyfill is opt-in)
function ensureInterface() {
    if (typeof navigator.credentials.create !== "function") {
        debug = console.log;

        return loadJavaScript("/webauthn/webauthn-polyfill/webauthn-polyfill.js")
            .then(() => {
                return loadJavaScript("/webauthn/webauthn-soft-authn/soft-authn.js");
            });
    } else {
        return Promise.resolve();
    }
}

function loadJavaScript(path) {
    return new Promise((resolve, reject) => {
        // dynamic loading of polyfill script by creating new <script> tag and seeing the src=
        var scriptElem = document.createElement("script");
        if (typeof scriptElem !== "object") {
            debug("ensureInterface: Error creating script element while attempting loading polyfill");
            return reject(new Error("ensureInterface: Error creating script element while loading polyfill"));
        }
        scriptElem.type = "application/javascript";
        scriptElem.onload = function() {
            debug("!!! Loaded " + path + " ...");
            return resolve();
        };
        scriptElem.onerror = function() {
            debug("navigator.credentials.create does not exist");
            resolve();
        };
        scriptElem.src = path;
        if (document.body) {
            document.body.appendChild(scriptElem);
        } else {
            debug("ensureInterface: DOM has no body");
            return reject(new Error("ensureInterface: DOM has no body"));
        }
    });
}

function standardSetup(cb) {
    return ensureInterface()
        .then(() => {
            if (cb) return cb();
        });
}
//************* END DELETE AFTER 1/1/2018 *************** //

/* JSHINT */
/* globals promise_rejects, assert_class_string, assert_equals, assert_idl_attribute, assert_readonly, promise_test */
/* exported standardSetup, CreateCredentialsTest, GetCredentialsTest */