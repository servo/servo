/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var count = 0;

function testCaller(obj) {
    switch (++count) {
      case 1:
      case 2:
        /*
         * The first two times, obj is objA. The first time, we reference
         * arguments.callee.caller before obj.go, so the caller getter must
         * force the joined function object in the stack frame to cross the
         * method read barrier. The second time, obj.go has been cloned and
         * it should match the new frame's callee from the get-go.
         */
        assert.sameValue(obj, objA);
        break;

      case 3: {
        assert.sameValue(obj, objB);

        /*
         * Store another clone of the joined function object before obj.go has
         * been read, but after it has been invoked via objB.go(objB).
         *
         * In this case, arguments.callee.caller must not lie and return what
         * is currently stored in objB.go, since that function object (objA.go)
         * was cloned earlier, when count was 1, and it is not the function
         * object that was truly invoked.
         *
         * But since the invocation of objB.go(objB) did not clone go, and the
         * following assignment overwrote the invoked value, leaving the only
         * reference to the joined function object for go in the stack frame's
         * callee (argv[-2]) member, the arguments.callee.caller reference must
         * clone a function object for the callee, store it as the callee, and
         * return it here.
         *
         * It won't equal obj.go, but (implementation detail) it should have
         * the same proto as obj.go
         */
        obj.go = objA.go;

        let caller = arguments.callee.caller;
        let obj_go = obj.go;
        return caller != obj_go && caller.__proto__ == obj_go.__proto__;
      }

      case 4: {
        assert.sameValue(obj, objC);

        let save = obj.go;
        delete obj.go;
        return arguments.callee.caller == save;
      }

      case 5: {
        assert.sameValue(obj, objD);

        let read = obj.go;
        break;
      }
    }

    return arguments.callee.caller == obj.go;
}

function make() {
    return {
        go: function(obj) {
            return testCaller(obj);
        }
    };
}

var objA = make(),
    objB = make(),
    objC = make(),
    objD = make();

assert.sameValue(true, objA.go(objA), "1");
assert.sameValue(true, objA.go(objA), "2");
assert.sameValue(true, objB.go(objB), "3");
assert.sameValue(true, objC.go(objC), "4");
assert.sameValue(true, objD.go(objD), "5");
