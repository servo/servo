let cyclicArray = [1];
cyclicArray.push(cyclicArray);

let cyclicObject = {key1: "data"};
cyclicObject.key2 = cyclicObject;

let cyclicSet = new Set([1]);
cyclicSet.add(cyclicSet);

let cyclicMap = new Map([["key1", 1]]);
cyclicMap.set("key2", cyclicMap);

const objects = {
    "null": {input: null},
    "undefined": {input: undefined},
    "int": {input: 1},
    "Infinity": {input: Infinity},
    "-Infinity": {input: -Infinity},
    "NaN": {input: NaN},
    "string": {input: "foo"},
    "true": {input: true},
    "false": {input: false},
    "bigint": {input: 1n},
    "RegExp": {input: /abc/g},
    "Date": {input: new Date('December 17, 1995 03:24:00')},
    "Error": {"input": new Error("message")},
    "TypeError": {"input": new TypeError("TypeError message")},
    "array": {input: [1,"foo"], output: [1, "foo"]},
    "nested array": {input: [1,[2]]},
    "set": {input: new Set([1, "foo", null])},
    "object": {input: {key1: 1, key2: false}},
    "nested object": {input: {key1: 1, key2: false}},
    "map": {input: new Map([[1, 1], ["key2", false]])},
    "cyclic array": {input: cyclicArray},
    "cyclic object": {input: cyclicObject},
    "cyclic map": {input: cyclicMap},
};
