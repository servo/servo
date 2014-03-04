/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

interface TestBinding {
           attribute boolean booleanAttribute;
           attribute byte byteAttribute;
           attribute octet octetAttribute;
           attribute short shortAttribute;
           attribute unsigned short unsignedShortAttribute;
           attribute long longAttribute;
           attribute unsigned long unsignedLongAttribute;
           attribute long long longLongAttribute;
           attribute unsigned long long unsignedLongLongAttribute;
           attribute float floatAttribute;
           attribute double doubleAttribute;

           attribute boolean? booleanAttributeNullable;
           attribute byte? byteAttributeNullable;
           attribute octet? octetAttributeNullable;
           attribute short? shortAttributeNullable;
           attribute unsigned short? unsignedShortAttributeNullable;
           attribute long? longAttributeNullable;
           attribute unsigned long? unsignedLongAttributeNullable;
           attribute long long? longLongAttributeNullable;
           attribute unsigned long long? unsignedLongLongAttributeNullable;
           attribute float? floatAttributeNullable;
           attribute double? doubleAttributeNullable;

  // FIXME (issue #1813) Doesn't currently compile.
  // void passOptionalBoolean(optional boolean arg);
  // void passOptionalByte(optional byte arg);
  // void passOptionalOctet(optional octet arg);
  // void passOptionalShort(optional short arg);
  // void passOptionalUnsignedShort(optional unsigned short arg);
  // void passOptionalLong(optional long arg);
  // void passOptionalUnsignedLong(optional unsigned long arg);
  // void passOptionalLongLong(optional long long arg);
  // void passOptionalUnsignedLongLong(optional unsigned long long arg);
  // void passOptionalFloat(optional float arg);
  // void passOptionalDouble(optional double arg);

  void passOptionalBooleanWithDefault(optional boolean arg = false);
  void passOptionalByteWithDefault(optional byte arg = 0);
  void passOptionalOctetWithDefault(optional octet arg = 19);
  void passOptionalShortWithDefault(optional short arg = 5);
  void passOptionalUnsignedShortWithDefault(optional unsigned short arg = 2);
  void passOptionalLongWithDefault(optional long arg = 7);
  void passOptionalUnsignedLongWithDefault(optional unsigned long arg = 6);
  void passOptionalLongLongWithDefault(optional long long arg = -12);
  void passOptionalUnsignedLongLongWithDefault(optional unsigned long long arg = 17);

  void passOptionalNullableBooleanWithDefault(optional boolean? arg = null);
  void passOptionalNullableByteWithDefault(optional byte? arg = null);
  void passOptionalNullableOctetWithDefault(optional octet? arg = null);
  void passOptionalNullableShortWithDefault(optional short? arg = null);
  void passOptionalNullableUnsignedShortWithDefault(optional unsigned short? arg = null);
  void passOptionalNullableLongWithDefault(optional long? arg = null);
  void passOptionalNullableUnsignedLongWithDefault(optional unsigned long? arg = null);
  void passOptionalNullableLongLongWithDefault(optional long long? arg = null);
  void passOptionalNullableUnsignedLongLongWithDefault(optional unsigned long long? arg = null);
};
