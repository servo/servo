/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://webbluetoothcg.github.io/web-bluetooth/#characteristicproperties

interface BluetoothCharacteristicProperties {
    readonly attribute boolean broadcast;
    readonly attribute boolean read;
    readonly attribute boolean writeWithoutResponse;
    readonly attribute boolean write;
    readonly attribute boolean notify;
    readonly attribute boolean indicate;
    readonly attribute boolean authenticatedSignedWrites;
    readonly attribute boolean reliableWrite;
    readonly attribute boolean writableAuxiliaries;
};
