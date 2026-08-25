// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Instance is extensible
info: |
  Intl.DisplayNames ( locales , options )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let displayNames be ? OrdinaryCreateFromConstructor(NewTarget, "%DisplayNamesPrototype%",
    « [[InitializedDisplayNames]], [[Locale]], [[Style]], [[Type]], [[Fallback]], [[Fields]] »).
  ...
  12. Let type be ? GetOption(options, "type", "string", « "language", "region", "script", "currency" », undefined).
  13. If type is undefined, throw a TypeError exception.
  ...

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  ObjectCreate ( proto [ , internalSlotsList ] )

  ...
  2. Let obj be a newly created object with an internal slot for each name in internalSlotsList.
  3. Set obj's essential internal methods to the default ordinary object definitions specified in 9.1.
  4. Set obj.[[Prototype]] to proto.
  5. Set obj.[[Extensible]] to true.
  6. Return obj.
features: [Intl.DisplayNames]
---*/

var obj = new Intl.DisplayNames(undefined, {type: 'language'});

assert(Object.isExtensible(obj));
