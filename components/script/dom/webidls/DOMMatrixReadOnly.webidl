/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://drafts.fxtf.org/geometry-1/#DOMMatrix
 *
 * Copyright:
 * To the extent possible under law, the editors have waived all copyright and
 * related or neighboring rights to this work.
 */

[Constructor,
 // Constructor(DOMString transformList)
 Constructor(sequence<unrestricted double> numberSequence),
 Exposed=(Window,Worker)]
interface DOMMatrixReadOnly {

    [NewObject, Throws] static DOMMatrixReadOnly fromMatrix(optional DOMMatrixInit other);
//  [NewObject] static DOMMatrixReadOnly fromFloat32Array(Float32Array array32);
//  [NewObject] static DOMMatrixReadOnly fromFloat64Array(Float64Array array64);

    // These attributes are simple aliases for certain elements of the 4x4 matrix
    readonly attribute unrestricted double a;
    readonly attribute unrestricted double b;
    readonly attribute unrestricted double c;
    readonly attribute unrestricted double d;
    readonly attribute unrestricted double e;
    readonly attribute unrestricted double f;

    readonly attribute unrestricted double m11;
    readonly attribute unrestricted double m12;
    readonly attribute unrestricted double m13;
    readonly attribute unrestricted double m14;
    readonly attribute unrestricted double m21;
    readonly attribute unrestricted double m22;
    readonly attribute unrestricted double m23;
    readonly attribute unrestricted double m24;
    readonly attribute unrestricted double m31;
    readonly attribute unrestricted double m32;
    readonly attribute unrestricted double m33;
    readonly attribute unrestricted double m34;
    readonly attribute unrestricted double m41;
    readonly attribute unrestricted double m42;
    readonly attribute unrestricted double m43;
    readonly attribute unrestricted double m44;

    readonly attribute boolean is2D;
    readonly attribute boolean isIdentity;

    // Immutable transform methods
    DOMMatrix translate(optional unrestricted double tx = 0,
                        optional unrestricted double ty = 0,
                        optional unrestricted double tz = 0);
    DOMMatrix scale(optional unrestricted double scaleX = 1,
                    optional unrestricted double scaleY,
                    optional unrestricted double scaleZ = 1,
                    optional unrestricted double originX = 0,
                    optional unrestricted double originY = 0,
                    optional unrestricted double originZ = 0);
    DOMMatrix scale3d(optional unrestricted double scale = 1,
                      optional unrestricted double originX = 0,
                      optional unrestricted double originY = 0,
                      optional unrestricted double originZ = 0);
    DOMMatrix rotate(optional unrestricted double rotX = 0,
                     optional unrestricted double rotY,
                     optional unrestricted double rotZ);
    DOMMatrix rotateFromVector(optional unrestricted double x = 0,
                               optional unrestricted double y = 0);
    DOMMatrix rotateAxisAngle(optional unrestricted double x = 0,
                              optional unrestricted double y = 0,
                              optional unrestricted double z = 0,
                              optional unrestricted double angle = 0);
    DOMMatrix skewX(optional unrestricted double sx = 0);
    DOMMatrix skewY(optional unrestricted double sy = 0);
    [Throws] DOMMatrix multiply(optional DOMMatrixInit other);
    DOMMatrix flipX();
    DOMMatrix flipY();
    DOMMatrix inverse();

    DOMPoint            transformPoint(optional DOMPointInit point);
//  Float32Array        toFloat32Array();
//  Float64Array        toFloat64Array();
//                      stringifier;
//                      serializer = { attribute };

};
