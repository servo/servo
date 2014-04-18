/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

enum TestEnum { "", "foo", "bar" };

/* dictionary TestDictionary {
  // boolean booleanValue;
  // byte byteValue;
  // octet octetValue;
  // short shortValue;
  // unsigned short unsignedShortValue;
  // long longValue;
  // unsigned long unsignedLongValue;
  // long long longLongValue;
  // unsigned long long unsignedLongLongValue;
  // float floatValue;
  // double doubleValue;
  // DOMString stringValue;
  // TestEnum enumValue;
  // Blob interfaceValue;
  // any anyValue;
}; */

dictionary TestDictionaryDefaults {
  // boolean booleanValue = false;
  // byte byteValue = 7;
  // octet octetValue = 7;
  short shortValue = 7;
  unsigned short unsignedShortValue = 7;
  long longValue = 7;
  unsigned long unsignedLongValue = 7;
  // long long longLongValue = 7;
  // unsigned long long unsignedLongLongValue = 7;
  // float floatValue = 7.0;
  // double doubleValue = 7.0;
  DOMString stringValue = "";
  // TestEnum enumValue = "bar";

  // boolean? nullableBooleanValue = false;
  // byte? nullableByteValue = 7;
  // octet? nullableOctetValue = 7;
  short? nullableShortValue = 7;
  unsigned short? nullableUnsignedShortValue = 7;
  long? nullableLongValue = 7;
  unsigned long? nullableUnsignedLongValue = 7;
  // long long? nullableLongLongValue = 7;
  // unsigned long long? nullableUnsignedLongLongValue = 7;
  // float? nullableFloatValue = 7.0;
  // double? nullableDoubleValue = 7.0;
  DOMString? nullableStringValue = "";
  // TestEnum? nullableEnumValue = "bar";
};

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
           attribute DOMString stringAttribute;
           attribute TestEnum enumAttribute;
           attribute Blob interfaceAttribute;
           attribute any anyAttribute;

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
           attribute DOMString? stringAttributeNullable;
  readonly attribute TestEnum? enumAttributeNullable;
           attribute Blob? interfaceAttributeNullable;

  void passOptionalBoolean(optional boolean arg);
  void passOptionalByte(optional byte arg);
  void passOptionalOctet(optional octet arg);
  void passOptionalShort(optional short arg);
  void passOptionalUnsignedShort(optional unsigned short arg);
  void passOptionalLong(optional long arg);
  void passOptionalUnsignedLong(optional unsigned long arg);
  void passOptionalLongLong(optional long long arg);
  void passOptionalUnsignedLongLong(optional unsigned long long arg);
  void passOptionalFloat(optional float arg);
  void passOptionalDouble(optional double arg);
  void passOptionalString(optional DOMString arg);
  // void passOptionalEnum(optional TestEnum arg);
  void passOptionalInterface(optional Blob arg);
  void passOptionalAny(optional any arg);

  void passOptionalNullableBoolean(optional boolean? arg);
  void passOptionalNullableByte(optional byte? arg);
  void passOptionalNullableOctet(optional octet? arg);
  void passOptionalNullableShort(optional short? arg);
  void passOptionalNullableUnsignedShort(optional unsigned short? arg);
  void passOptionalNullableLong(optional long? arg);
  void passOptionalNullableUnsignedLong(optional unsigned long? arg);
  void passOptionalNullableLongLong(optional long long? arg);
  void passOptionalNullableUnsignedLongLong(optional unsigned long long? arg);
  void passOptionalNullableFloat(optional float? arg);
  void passOptionalNullableDouble(optional double? arg);
  void passOptionalNullableString(optional DOMString? arg);
  // void passOptionalNullableEnum(optional TestEnum? arg);
  // void passOptionalNullableInterface(optional Blob? arg);

  void passOptionalBooleanWithDefault(optional boolean arg = false);
  void passOptionalByteWithDefault(optional byte arg = 0);
  void passOptionalOctetWithDefault(optional octet arg = 19);
  void passOptionalShortWithDefault(optional short arg = 5);
  void passOptionalUnsignedShortWithDefault(optional unsigned short arg = 2);
  void passOptionalLongWithDefault(optional long arg = 7);
  void passOptionalUnsignedLongWithDefault(optional unsigned long arg = 6);
  void passOptionalLongLongWithDefault(optional long long arg = -12);
  void passOptionalUnsignedLongLongWithDefault(optional unsigned long long arg = 17);
  void passOptionalStringWithDefault(optional DOMString arg = "");
  void passOptionalEnumWithDefault(optional TestEnum arg = "foo");

  void passOptionalNullableBooleanWithDefault(optional boolean? arg = null);
  void passOptionalNullableByteWithDefault(optional byte? arg = null);
  void passOptionalNullableOctetWithDefault(optional octet? arg = null);
  void passOptionalNullableShortWithDefault(optional short? arg = null);
  void passOptionalNullableUnsignedShortWithDefault(optional unsigned short? arg = null);
  void passOptionalNullableLongWithDefault(optional long? arg = null);
  void passOptionalNullableUnsignedLongWithDefault(optional unsigned long? arg = null);
  void passOptionalNullableLongLongWithDefault(optional long long? arg = null);
  void passOptionalNullableUnsignedLongLongWithDefault(optional unsigned long long? arg = null);
  void passOptionalNullableStringWithDefault(optional DOMString? arg = null);
  // void passOptionalNullableEnumWithDefault(optional TestEnum? arg = null);
  void passOptionalNullableInterfaceWithDefault(optional Blob? arg = null);
  void passOptionalAnyWithDefault(optional any arg = null);

  void passOptionalNullableBooleanWithNonNullDefault(optional boolean? arg = false);
  void passOptionalNullableByteWithNonNullDefault(optional byte? arg = 7);
  void passOptionalNullableOctetWithNonNullDefault(optional octet? arg = 7);
  void passOptionalNullableShortWithNonNullDefault(optional short? arg = 7);
  void passOptionalNullableUnsignedShortWithNonNullDefault(optional unsigned short? arg = 7);
  void passOptionalNullableLongWithNonNullDefault(optional long? arg = 7);
  void passOptionalNullableUnsignedLongWithNonNullDefault(optional unsigned long? arg = 7);
  void passOptionalNullableLongLongWithNonNullDefault(optional long long? arg = 7);
  void passOptionalNullableUnsignedLongLongWithNonNullDefault(optional unsigned long long? arg = 7);
  // void passOptionalNullableFloatWithNonNullDefault(optional float? arg = 0.0);
  // void passOptionalNullableDoubleWithNonNullDefault(optional double? arg = 0.0);
  void passOptionalNullableStringWithNonNullDefault(optional DOMString? arg = "");
  // void passOptionalNullableEnumWithNonNullDefault(optional TestEnum? arg = "foo");
};
