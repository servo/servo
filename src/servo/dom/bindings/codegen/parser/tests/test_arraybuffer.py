import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestArrayBuffer {
          attribute ArrayBuffer bufferAttr;
          void bufferMethod(ArrayBuffer arg1, ArrayBuffer? arg2, ArrayBuffer[] arg3, sequence<ArrayBuffer> arg4);

          attribute ArrayBufferView viewAttr;
          void viewMethod(ArrayBufferView arg1, ArrayBufferView? arg2, ArrayBufferView[] arg3, sequence<ArrayBufferView> arg4);

          attribute Int8Array int8ArrayAttr;
          void int8ArrayMethod(Int8Array arg1, Int8Array? arg2, Int8Array[] arg3, sequence<Int8Array> arg4);

          attribute Uint8Array uint8ArrayAttr;
          void uint8ArrayMethod(Uint8Array arg1, Uint8Array? arg2, Uint8Array[] arg3, sequence<Uint8Array> arg4);

          attribute Uint8ClampedArray uint8ClampedArrayAttr;
          void uint8ClampedArrayMethod(Uint8ClampedArray arg1, Uint8ClampedArray? arg2, Uint8ClampedArray[] arg3, sequence<Uint8ClampedArray> arg4);

          attribute Int16Array int16ArrayAttr;
          void int16ArrayMethod(Int16Array arg1, Int16Array? arg2, Int16Array[] arg3, sequence<Int16Array> arg4);

          attribute Uint16Array uint16ArrayAttr;
          void uint16ArrayMethod(Uint16Array arg1, Uint16Array? arg2, Uint16Array[] arg3, sequence<Uint16Array> arg4);

          attribute Int32Array int32ArrayAttr;
          void int32ArrayMethod(Int32Array arg1, Int32Array? arg2, Int32Array[] arg3, sequence<Int32Array> arg4);

          attribute Uint32Array uint32ArrayAttr;
          void uint32ArrayMethod(Uint32Array arg1, Uint32Array? arg2, Uint32Array[] arg3, sequence<Uint32Array> arg4);

          attribute Float32Array float32ArrayAttr;
          void float32ArrayMethod(Float32Array arg1, Float32Array? arg2, Float32Array[] arg3, sequence<Float32Array> arg4);

          attribute Float64Array float64ArrayAttr;
          void float64ArrayMethod(Float64Array arg1, Float64Array? arg2, Float64Array[] arg3, sequence<Float64Array> arg4);
        };
    """)

    results = parser.finish()

    iface = results[0]

    harness.ok(True, "TestArrayBuffer interface parsed without error")
    harness.check(len(iface.members), 22, "Interface should have twenty two members")

    members = iface.members

    def checkStuff(attr, method, t):
        harness.ok(isinstance(attr, WebIDL.IDLAttribute), "Expect an IDLAttribute")
        harness.ok(isinstance(method, WebIDL.IDLMethod), "Expect an IDLMethod")

        harness.check(str(attr.type), t, "Expect an ArrayBuffer type")
        harness.ok(attr.type.isSpiderMonkeyInterface(), "Should test as a js interface")

        (retType, arguments) = method.signatures()[0]
        harness.ok(retType.isVoid(), "Should have a void return type")
        harness.check(len(arguments), 4, "Expect 4 arguments")

        harness.check(str(arguments[0].type), t,  "Expect an ArrayBuffer type")
        harness.ok(arguments[0].type.isSpiderMonkeyInterface(), "Should test as a js interface")

        harness.check(str(arguments[1].type), t + "OrNull",  "Expect an ArrayBuffer type")
        harness.ok(arguments[1].type.inner.isSpiderMonkeyInterface(), "Should test as a js interface")

        harness.check(str(arguments[2].type), t + "Array",  "Expect an ArrayBuffer type")
        harness.ok(arguments[2].type.inner.isSpiderMonkeyInterface(), "Should test as a js interface")

        harness.check(str(arguments[3].type), t + "Sequence",  "Expect an ArrayBuffer type")
        harness.ok(arguments[3].type.inner.isSpiderMonkeyInterface(), "Should test as a js interface")


    checkStuff(members[0],  members[1],  "ArrayBuffer")
    checkStuff(members[2],  members[3],  "ArrayBufferView")
    checkStuff(members[4],  members[5],  "Int8Array")
    checkStuff(members[6],  members[7],  "Uint8Array")
    checkStuff(members[8],  members[9],  "Uint8ClampedArray")
    checkStuff(members[10], members[11], "Int16Array")
    checkStuff(members[12], members[13], "Uint16Array")
    checkStuff(members[14], members[15], "Int32Array")
    checkStuff(members[16], members[17], "Uint32Array")
    checkStuff(members[18], members[19], "Float32Array")
    checkStuff(members[20], members[21], "Float64Array")
