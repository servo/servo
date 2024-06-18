/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.

enum TestEnum { "", "foo", "bar" };
typedef (DOMString or URL or Blob) TestTypedef;
typedef (DOMString or URL or Blob)? TestTypedefNullableUnion;
typedef DOMString TestTypedefString;
typedef Blob TestTypedefInterface;

dictionary TestDictionary {
  required boolean requiredValue;
  boolean booleanValue;
  byte byteValue;
  octet octetValue;
  short shortValue;
  unsigned short unsignedShortValue;
  long longValue;
  unsigned long unsignedLongValue;
  long long longLongValue;
  unsigned long long unsignedLongLongValue;
  unrestricted float unrestrictedFloatValue;
  float floatValue;
  unrestricted double unrestrictedDoubleValue;
  double doubleValue;
  DOMString stringValue;
  USVString usvstringValue;
  TestEnum enumValue;
  Blob interfaceValue;
  any anyValue;
  object objectValue;
  TestDictionaryDefaults dict = {};
  sequence<TestDictionaryDefaults> seqDict;
  // Testing codegen to import Element correctly, ensure no other code references Element directly
  sequence<Element> elementSequence;
  // Reserved rust keyword
  DOMString type;
  // These are used to test bidirectional conversion
  // and differentiation of non-required and nullable types
  // in dictionaries.
  DOMString? nonRequiredNullable;
  DOMString? nonRequiredNullable2;
  SimpleCallback noCallbackImport;
  callbackWithOnlyOneOptionalArg noCallbackImport2;
};

dictionary TestDictionaryParent {
  DOMString parentStringMember;
};

dictionary TestDictionaryWithParent : TestDictionaryParent {
  DOMString stringMember;
};

dictionary TestDictionaryDefaults {
  boolean booleanValue = false;
  byte byteValue = 7;
  octet octetValue = 7;
  short shortValue = 7;
  unsigned short unsignedShortValue = 7;
  long longValue = 7;
  unsigned long unsignedLongValue = 7;
  long long longLongValue = 7;
  unsigned long long unsignedLongLongValue = 7;
  unrestricted float unrestrictedFloatValue = 7.0;
  float floatValue = 7.0;
  unrestricted double UnrestrictedDoubleValue = 7.0;
  double doubleValue = 7.0;
  ByteString bytestringValue = "foo";
  DOMString stringValue = "foo";
  USVString usvstringValue = "foo";
  TestEnum enumValue = "bar";
  any anyValue = null;
  sequence<object> arrayValue = [];

  boolean? nullableBooleanValue = false;
  byte? nullableByteValue = 7;
  octet? nullableOctetValue = 7;
  short? nullableShortValue = 7;
  unsigned short? nullableUnsignedShortValue = 7;
  long? nullableLongValue = 7;
  unsigned long? nullableUnsignedLongValue = 7;
  long long? nullableLongLongValue = 7;
  unsigned long long? nullableUnsignedLongLongValue = 7;
  unrestricted float? nullableUnrestrictedFloatValue = 7.0;
  float? nullableFloatValue = 7.0;
  unrestricted double? nullableUnrestrictedDoubleValue = 7.0;
  double? nullableDoubleValue = 7.0;
  ByteString? nullableBytestringValue = "foo";
  DOMString? nullableStringValue = "foo";
  USVString? nullableUsvstringValue = "foo";
  // TestEnum? nullableEnumValue = "bar";
  object? nullableObjectValue = null;
};

dictionary TestURLLike {
  required DOMString href;
};

[Pref="dom.testbinding.enabled",
 Exposed=(Window,Worker)
]
interface TestBinding {
           [Throws] constructor();
           [Throws] constructor(sequence<unrestricted double> numberSequence);
           [Throws] constructor(unrestricted double num);
           attribute boolean booleanAttribute;
           attribute byte byteAttribute;
           attribute octet octetAttribute;
           attribute short shortAttribute;
           attribute unsigned short unsignedShortAttribute;
           attribute long longAttribute;
           attribute unsigned long unsignedLongAttribute;
           attribute long long longLongAttribute;
           attribute unsigned long long unsignedLongLongAttribute;
           attribute unrestricted float unrestrictedFloatAttribute;
           attribute float floatAttribute;
           attribute unrestricted double unrestrictedDoubleAttribute;
           attribute double doubleAttribute;
           attribute DOMString stringAttribute;
           attribute USVString usvstringAttribute;
           attribute ByteString byteStringAttribute;
           attribute TestEnum enumAttribute;
           attribute Blob interfaceAttribute;
           attribute (HTMLElement or long) unionAttribute;
           attribute (Event or DOMString) union2Attribute;
           attribute (Event or USVString) union3Attribute;
           attribute (DOMString or unsigned long) union4Attribute;
           attribute (DOMString or boolean) union5Attribute;
           attribute (unsigned long or boolean) union6Attribute;
           attribute (Blob or boolean) union7Attribute;
           attribute (Blob or unsigned long) union8Attribute;
           attribute (ByteString or long) union9Attribute;
  readonly attribute Uint8ClampedArray arrayAttribute;
           attribute any anyAttribute;
           attribute object objectAttribute;

           attribute boolean? booleanAttributeNullable;
           attribute byte? byteAttributeNullable;
           attribute octet? octetAttributeNullable;
           attribute short? shortAttributeNullable;
           attribute unsigned short? unsignedShortAttributeNullable;
           attribute long? longAttributeNullable;
           attribute unsigned long? unsignedLongAttributeNullable;
           attribute long long? longLongAttributeNullable;
           attribute unsigned long long? unsignedLongLongAttributeNullable;
           attribute unrestricted float? unrestrictedFloatAttributeNullable;
           attribute float? floatAttributeNullable;
           attribute unrestricted double? unrestrictedDoubleAttributeNullable;
           attribute double? doubleAttributeNullable;
           attribute DOMString? stringAttributeNullable;
           attribute USVString? usvstringAttributeNullable;
           attribute ByteString? byteStringAttributeNullable;
  readonly attribute TestEnum? enumAttributeNullable;
           attribute Blob? interfaceAttributeNullable;
           attribute URL? interfaceAttributeWeak;
           attribute object? objectAttributeNullable;
           attribute (HTMLElement or long)? unionAttributeNullable;
           attribute (Event or DOMString)? union2AttributeNullable;
           attribute (Blob or boolean)? union3AttributeNullable;
           attribute (unsigned long or boolean)? union4AttributeNullable;
           attribute (DOMString or boolean)? union5AttributeNullable;
           attribute (ByteString or long)? union6AttributeNullable;
  [BinaryName="BinaryRenamedAttribute"] attribute DOMString attrToBinaryRename;
  [BinaryName="BinaryRenamedAttribute2"] attribute DOMString attr-to-binary-rename;
  attribute DOMString attr-to-automatically-rename;

  const long long constantInt64 = -1;
  const unsigned long long constantUint64 = 1;
  const float constantFloat32 = 1.0;
  const double constantFloat64 = 1.0;
  const unrestricted float constantUnrestrictedFloat32 = 1.0;
  const unrestricted double constantUnrestrictedFloat64 = 1.0;

  [PutForwards=booleanAttribute]
  readonly attribute TestBinding forwardedAttribute;

  [BinaryName="BinaryRenamedMethod"] undefined methToBinaryRename();
  undefined receiveVoid();
  boolean receiveBoolean();
  byte receiveByte();
  octet receiveOctet();
  short receiveShort();
  unsigned short receiveUnsignedShort();
  long receiveLong();
  unsigned long receiveUnsignedLong();
  long long receiveLongLong();
  unsigned long long receiveUnsignedLongLong();
  unrestricted float receiveUnrestrictedFloat();
  float receiveFloat();
  unrestricted double receiveUnrestrictedDouble();
  double receiveDouble();
  DOMString receiveString();
  USVString receiveUsvstring();
  ByteString receiveByteString();
  TestEnum receiveEnum();
  Blob receiveInterface();
  any receiveAny();
  object receiveObject();
  (HTMLElement or long) receiveUnion();
  (Event or DOMString) receiveUnion2();
  (DOMString or sequence<long>) receiveUnion3();
  (DOMString or sequence<DOMString>) receiveUnion4();
  (Blob or sequence<Blob>) receiveUnion5();
  (DOMString or unsigned long) receiveUnion6();
  (DOMString or boolean) receiveUnion7();
  (unsigned long or boolean) receiveUnion8();
  (HTMLElement or unsigned long or DOMString or boolean) receiveUnion9();
  (ByteString or long) receiveUnion10();
  (sequence<ByteString> or long or DOMString) receiveUnion11();
  sequence<long> receiveSequence();
  sequence<Blob> receiveInterfaceSequence();

  byte? receiveNullableByte();
  boolean? receiveNullableBoolean();
  octet? receiveNullableOctet();
  short? receiveNullableShort();
  unsigned short? receiveNullableUnsignedShort();
  long? receiveNullableLong();
  unsigned long? receiveNullableUnsignedLong();
  long long? receiveNullableLongLong();
  unsigned long long? receiveNullableUnsignedLongLong();
  unrestricted float? receiveNullableUnrestrictedFloat();
  float? receiveNullableFloat();
  unrestricted double? receiveNullableUnrestrictedDouble();
  double? receiveNullableDouble();
  DOMString? receiveNullableString();
  USVString? receiveNullableUsvstring();
  ByteString? receiveNullableByteString();
  TestEnum? receiveNullableEnum();
  Blob? receiveNullableInterface();
  object? receiveNullableObject();
  (HTMLElement or long)? receiveNullableUnion();
  (Event or DOMString)? receiveNullableUnion2();
  (DOMString or sequence<long>)? receiveNullableUnion3();
  (sequence<long> or boolean)? receiveNullableUnion4();
  (unsigned long or boolean)? receiveNullableUnion5();
  (ByteString or long)? receiveNullableUnion6();
  sequence<long>? receiveNullableSequence();
  TestDictionary receiveTestDictionaryWithSuccessOnKeyword();
  boolean dictMatchesPassedValues(TestDictionary arg);

  (DOMString or object) receiveUnionIdentity((DOMString or object) arg);

  undefined passBoolean(boolean arg);
  undefined passByte(byte arg);
  undefined passOctet(octet arg);
  undefined passShort(short arg);
  undefined passUnsignedShort(unsigned short arg);
  undefined passLong(long arg);
  undefined passUnsignedLong(unsigned long arg);
  undefined passLongLong(long long arg);
  undefined passUnsignedLongLong(unsigned long long arg);
  undefined passUnrestrictedFloat(unrestricted float arg);
  undefined passFloat(float arg);
  undefined passUnrestrictedDouble(unrestricted double arg);
  undefined passDouble(double arg);
  undefined passString(DOMString arg);
  undefined passUsvstring(USVString arg);
  undefined passByteString(ByteString arg);
  undefined passEnum(TestEnum arg);
  undefined passInterface(Blob arg);
  undefined passTypedArray(Int8Array arg);
  undefined passTypedArray2(ArrayBuffer arg);
  undefined passTypedArray3(ArrayBufferView arg);
  undefined passUnion((HTMLElement or long) arg);
  undefined passUnion2((Event or DOMString) data);
  undefined passUnion3((Blob or DOMString) data);
  undefined passUnion4((DOMString or sequence<DOMString>) seq);
  undefined passUnion5((DOMString or boolean) data);
  undefined passUnion6((unsigned long or boolean) bool);
  undefined passUnion7((sequence<DOMString> or unsigned long) arg);
  undefined passUnion8((sequence<ByteString> or long) arg);
  undefined passUnion9((TestDictionary or long) arg);
  undefined passUnion10((DOMString or object) arg);
  undefined passUnion11((ArrayBuffer or ArrayBufferView) arg);
  undefined passUnionWithTypedef((Document or TestTypedef) arg);
  undefined passUnionWithTypedef2((sequence<long> or TestTypedef) arg);
  undefined passAny(any arg);
  undefined passObject(object arg);
  undefined passCallbackFunction(Function fun);
  undefined passCallbackInterface(EventListener listener);
  undefined passSequence(sequence<long> seq);
  undefined passAnySequence(sequence<any> seq);
  sequence<any> anySequencePassthrough(sequence<any> seq);
  undefined passObjectSequence(sequence<object> seq);
  undefined passStringSequence(sequence<DOMString> seq);
  undefined passInterfaceSequence(sequence<Blob> seq);

  undefined passOverloaded(ArrayBuffer arg);
  undefined passOverloaded(DOMString arg);

  // https://github.com/servo/servo/pull/26154
  DOMString passOverloadedDict(Node arg);
  DOMString passOverloadedDict(TestURLLike arg);

  undefined passNullableBoolean(boolean? arg);
  undefined passNullableByte(byte? arg);
  undefined passNullableOctet(octet? arg);
  undefined passNullableShort(short? arg);
  undefined passNullableUnsignedShort(unsigned short? arg);
  undefined passNullableLong(long? arg);
  undefined passNullableUnsignedLong(unsigned long? arg);
  undefined passNullableLongLong(long long? arg);
  undefined passNullableUnsignedLongLong(unsigned long long? arg);
  undefined passNullableUnrestrictedFloat(unrestricted float? arg);
  undefined passNullableFloat(float? arg);
  undefined passNullableUnrestrictedDouble(unrestricted double? arg);
  undefined passNullableDouble(double? arg);
  undefined passNullableString(DOMString? arg);
  undefined passNullableUsvstring(USVString? arg);
  undefined passNullableByteString(ByteString? arg);
  // void passNullableEnum(TestEnum? arg);
  undefined passNullableInterface(Blob? arg);
  undefined passNullableObject(object? arg);
  undefined passNullableTypedArray(Int8Array? arg);
  undefined passNullableUnion((HTMLElement or long)? arg);
  undefined passNullableUnion2((Event or DOMString)? data);
  undefined passNullableUnion3((DOMString or sequence<long>)? data);
  undefined passNullableUnion4((sequence<long> or boolean)? bool);
  undefined passNullableUnion5((unsigned long or boolean)? arg);
  undefined passNullableUnion6((ByteString or long)? arg);
  undefined passNullableCallbackFunction(Function? fun);
  undefined passNullableCallbackInterface(EventListener? listener);
  undefined passNullableSequence(sequence<long>? seq);

  undefined passOptionalBoolean(optional boolean arg);
  undefined passOptionalByte(optional byte arg);
  undefined passOptionalOctet(optional octet arg);
  undefined passOptionalShort(optional short arg);
  undefined passOptionalUnsignedShort(optional unsigned short arg);
  undefined passOptionalLong(optional long arg);
  undefined passOptionalUnsignedLong(optional unsigned long arg);
  undefined passOptionalLongLong(optional long long arg);
  undefined passOptionalUnsignedLongLong(optional unsigned long long arg);
  undefined passOptionalUnrestrictedFloat(optional unrestricted float arg);
  undefined passOptionalFloat(optional float arg);
  undefined passOptionalUnrestrictedDouble(optional unrestricted double arg);
  undefined passOptionalDouble(optional double arg);
  undefined passOptionalString(optional DOMString arg);
  undefined passOptionalUsvstring(optional USVString arg);
  undefined passOptionalByteString(optional ByteString arg);
  undefined passOptionalEnum(optional TestEnum arg);
  undefined passOptionalInterface(optional Blob arg);
  undefined passOptionalUnion(optional (HTMLElement or long) arg);
  undefined passOptionalUnion2(optional (Event or DOMString) data);
  undefined passOptionalUnion3(optional (DOMString or sequence<long>) arg);
  undefined passOptionalUnion4(optional (sequence<long> or boolean) data);
  undefined passOptionalUnion5(optional (unsigned long or boolean) bool);
  undefined passOptionalUnion6(optional (ByteString or long) arg);
  undefined passOptionalAny(optional any arg);
  undefined passOptionalObject(optional object arg);
  undefined passOptionalCallbackFunction(optional Function fun);
  undefined passOptionalCallbackInterface(optional EventListener listener);
  undefined passOptionalSequence(optional sequence<long> seq);

  undefined passOptionalNullableBoolean(optional boolean? arg);
  undefined passOptionalNullableByte(optional byte? arg);
  undefined passOptionalNullableOctet(optional octet? arg);
  undefined passOptionalNullableShort(optional short? arg);
  undefined passOptionalNullableUnsignedShort(optional unsigned short? arg);
  undefined passOptionalNullableLong(optional long? arg);
  undefined passOptionalNullableUnsignedLong(optional unsigned long? arg);
  undefined passOptionalNullableLongLong(optional long long? arg);
  undefined passOptionalNullableUnsignedLongLong(optional unsigned long long? arg);
  undefined passOptionalNullableUnrestrictedFloat(optional unrestricted float? arg);
  undefined passOptionalNullableFloat(optional float? arg);
  undefined passOptionalNullableUnrestrictedDouble(optional unrestricted double? arg);
  undefined passOptionalNullableDouble(optional double? arg);
  undefined passOptionalNullableString(optional DOMString? arg);
  undefined passOptionalNullableUsvstring(optional USVString? arg);
  undefined passOptionalNullableByteString(optional ByteString? arg);
  // void passOptionalNullableEnum(optional TestEnum? arg);
  undefined passOptionalNullableInterface(optional Blob? arg);
  undefined passOptionalNullableObject(optional object? arg);
  undefined passOptionalNullableUnion(optional (HTMLElement or long)? arg);
  undefined passOptionalNullableUnion2(optional (Event or DOMString)? data);
  undefined passOptionalNullableUnion3(optional (DOMString or sequence<long>)? arg);
  undefined passOptionalNullableUnion4(optional (sequence<long> or boolean)? data);
  undefined passOptionalNullableUnion5(optional (unsigned long or boolean)? bool);
  undefined passOptionalNullableUnion6(optional (ByteString or long)? arg);
  undefined passOptionalNullableCallbackFunction(optional Function? fun);
  undefined passOptionalNullableCallbackInterface(optional EventListener? listener);
  undefined passOptionalNullableSequence(optional sequence<long>? seq);

  undefined passOptionalBooleanWithDefault(optional boolean arg = false);
  undefined passOptionalByteWithDefault(optional byte arg = 0);
  undefined passOptionalOctetWithDefault(optional octet arg = 19);
  undefined passOptionalShortWithDefault(optional short arg = 5);
  undefined passOptionalUnsignedShortWithDefault(optional unsigned short arg = 2);
  undefined passOptionalLongWithDefault(optional long arg = 7);
  undefined passOptionalUnsignedLongWithDefault(optional unsigned long arg = 6);
  undefined passOptionalLongLongWithDefault(optional long long arg = -12);
  undefined passOptionalUnsignedLongLongWithDefault(optional unsigned long long arg = 17);
  undefined passOptionalBytestringWithDefault(optional ByteString arg = "x");
  undefined passOptionalStringWithDefault(optional DOMString arg = "x");
  undefined passOptionalUsvstringWithDefault(optional USVString arg = "x");
  undefined passOptionalEnumWithDefault(optional TestEnum arg = "foo");
  undefined passOptionalSequenceWithDefault(optional sequence<long> seq = []);
  // void passOptionalUnionWithDefault(optional (HTMLElement or long) arg = 9);
  // void passOptionalUnion2WithDefault(optional(Event or DOMString)? data = "foo");

  undefined passOptionalNullableBooleanWithDefault(optional boolean? arg = null);
  undefined passOptionalNullableByteWithDefault(optional byte? arg = null);
  undefined passOptionalNullableOctetWithDefault(optional octet? arg = null);
  undefined passOptionalNullableShortWithDefault(optional short? arg = null);
  undefined passOptionalNullableUnsignedShortWithDefault(optional unsigned short? arg = null);
  undefined passOptionalNullableLongWithDefault(optional long? arg = null);
  undefined passOptionalNullableUnsignedLongWithDefault(optional unsigned long? arg = null);
  undefined passOptionalNullableLongLongWithDefault(optional long long? arg = null);
  undefined passOptionalNullableUnsignedLongLongWithDefault(optional unsigned long long? arg = null);
  undefined passOptionalNullableStringWithDefault(optional DOMString? arg = null);
  undefined passOptionalNullableUsvstringWithDefault(optional USVString? arg = null);
  undefined passOptionalNullableByteStringWithDefault(optional ByteString? arg = null);
  // void passOptionalNullableEnumWithDefault(optional TestEnum? arg = null);
  undefined passOptionalNullableInterfaceWithDefault(optional Blob? arg = null);
  undefined passOptionalNullableObjectWithDefault(optional object? arg = null);
  undefined passOptionalNullableUnionWithDefault(optional (HTMLElement or long)? arg = null);
  undefined passOptionalNullableUnion2WithDefault(optional (Event or DOMString)? data = null);
  // void passOptionalNullableCallbackFunctionWithDefault(optional Function? fun = null);
  undefined passOptionalNullableCallbackInterfaceWithDefault(optional EventListener? listener = null);
  undefined passOptionalAnyWithDefault(optional any arg = null);

  undefined passOptionalNullableBooleanWithNonNullDefault(optional boolean? arg = false);
  undefined passOptionalNullableByteWithNonNullDefault(optional byte? arg = 7);
  undefined passOptionalNullableOctetWithNonNullDefault(optional octet? arg = 7);
  undefined passOptionalNullableShortWithNonNullDefault(optional short? arg = 7);
  undefined passOptionalNullableUnsignedShortWithNonNullDefault(optional unsigned short? arg = 7);
  undefined passOptionalNullableLongWithNonNullDefault(optional long? arg = 7);
  undefined passOptionalNullableUnsignedLongWithNonNullDefault(optional unsigned long? arg = 7);
  undefined passOptionalNullableLongLongWithNonNullDefault(optional long long? arg = 7);
  undefined passOptionalNullableUnsignedLongLongWithNonNullDefault(optional unsigned long long? arg = 7);
  // void passOptionalNullableUnrestrictedFloatWithNonNullDefault(optional unrestricted float? arg = 0.0);
  // void passOptionalNullableFloatWithNonNullDefault(optional float? arg = 0.0);
  // void passOptionalNullableUnrestrictedDoubleWithNonNullDefault(optional unrestricted double? arg = 0.0);
  // void passOptionalNullableDoubleWithNonNullDefault(optional double? arg = 0.0);
  undefined passOptionalNullableStringWithNonNullDefault(optional DOMString? arg = "x");
  undefined passOptionalNullableUsvstringWithNonNullDefault(optional USVString? arg = "x");
  // void passOptionalNullableEnumWithNonNullDefault(optional TestEnum? arg = "foo");
  // void passOptionalNullableUnionWithNonNullDefault(optional (HTMLElement or long)? arg = 7);
  // void passOptionalNullableUnion2WithNonNullDefault(optional (Event or DOMString)? data = "foo");
  TestBinding passOptionalOverloaded(TestBinding arg0, optional unsigned long arg1 = 0,
                                     optional unsigned long arg2 = 0);
  undefined passOptionalOverloaded(Blob arg0, optional unsigned long arg1 = 0);

  undefined passVariadicBoolean(boolean... args);
  undefined passVariadicBooleanAndDefault(optional boolean arg = true, boolean... args);
  undefined passVariadicByte(byte... args);
  undefined passVariadicOctet(octet... args);
  undefined passVariadicShort(short... args);
  undefined passVariadicUnsignedShort(unsigned short... args);
  undefined passVariadicLong(long... args);
  undefined passVariadicUnsignedLong(unsigned long... args);
  undefined passVariadicLongLong(long long... args);
  undefined passVariadicUnsignedLongLong(unsigned long long... args);
  undefined passVariadicUnrestrictedFloat(unrestricted float... args);
  undefined passVariadicFloat(float... args);
  undefined passVariadicUnrestrictedDouble(unrestricted double... args);
  undefined passVariadicDouble(double... args);
  undefined passVariadicString(DOMString... args);
  undefined passVariadicUsvstring(USVString... args);
  undefined passVariadicByteString(ByteString... args);
  undefined passVariadicEnum(TestEnum... args);
  undefined passVariadicInterface(Blob... args);
  undefined passVariadicUnion((HTMLElement or long)... args);
  undefined passVariadicUnion2((Event or DOMString)... args);
  undefined passVariadicUnion3((Blob or DOMString)... args);
  undefined passVariadicUnion4((Blob or boolean)... args);
  undefined passVariadicUnion5((DOMString or unsigned long)... args);
  undefined passVariadicUnion6((unsigned long or boolean)... args);
  undefined passVariadicUnion7((ByteString or long)... args);
  undefined passVariadicAny(any... args);
  undefined passVariadicObject(object... args);

  undefined passSequenceSequence(sequence<sequence<long>> seq);
  sequence<sequence<long>> returnSequenceSequence();
  undefined passUnionSequenceSequence((long or sequence<sequence<long>>) seq);

  undefined passRecord(record<DOMString, long> arg);
  undefined passRecordWithUSVStringKey(record<USVString, long> arg);
  undefined passRecordWithByteStringKey(record<ByteString, long> arg);
  undefined passNullableRecord(record<DOMString, long>? arg);
  undefined passRecordOfNullableInts(record<DOMString, long?> arg);
  undefined passOptionalRecordOfNullableInts(optional record<DOMString, long?> arg);
  undefined passOptionalNullableRecordOfNullableInts(optional record<DOMString, long?>? arg);
  undefined passCastableObjectRecord(record<DOMString, TestBinding> arg);
  undefined passNullableCastableObjectRecord(record<DOMString, TestBinding?> arg);
  undefined passCastableObjectNullableRecord(record<DOMString, TestBinding>? arg);
  undefined passNullableCastableObjectNullableRecord(record<DOMString, TestBinding?>? arg);
  undefined passOptionalRecord(optional record<DOMString, long> arg);
  undefined passOptionalNullableRecord(optional record<DOMString, long>? arg);
  undefined passOptionalNullableRecordWithDefaultValue(optional record<DOMString, long>? arg = null);
  undefined passOptionalObjectRecord(optional record<DOMString, TestBinding> arg);
  undefined passStringRecord(record<DOMString, DOMString> arg);
  undefined passByteStringRecord(record<DOMString, ByteString> arg);
  undefined passRecordOfRecords(record<DOMString, record<DOMString, long>> arg);

  undefined passRecordUnion((long or record<DOMString, ByteString>) init);
  undefined passRecordUnion2((TestBinding or record<DOMString, ByteString>) init);
  undefined passRecordUnion3((TestBinding or sequence<sequence<ByteString>> or record<DOMString, ByteString>) init);

  record<DOMString, long> receiveRecord();
  record<USVString, long> receiveRecordWithUSVStringKey();
  record<ByteString, long> receiveRecordWithByteStringKey();
  record<DOMString, long>? receiveNullableRecord();
  record<DOMString, long?> receiveRecordOfNullableInts();
  record<DOMString, long?>? receiveNullableRecordOfNullableInts();
  record<DOMString, record<DOMString, long>> receiveRecordOfRecords();
  record<DOMString, any> receiveAnyRecord();

  static attribute boolean booleanAttributeStatic;
  static undefined receiveVoidStatic();
  boolean BooleanMozPreference(DOMString pref_name);
  DOMString StringMozPreference(DOMString pref_name);

  [Pref="dom.testbinding.prefcontrolled.enabled"]
  readonly attribute boolean prefControlledAttributeDisabled;
  [Pref="dom.testbinding.prefcontrolled.enabled"]
  static readonly attribute boolean prefControlledStaticAttributeDisabled;
  [Pref="dom.testbinding.prefcontrolled.enabled"]
  undefined prefControlledMethodDisabled();
  [Pref="dom.testbinding.prefcontrolled.enabled"]
  static undefined prefControlledStaticMethodDisabled();
  [Pref="dom.testbinding.prefcontrolled.enabled"]
  const unsigned short prefControlledConstDisabled = 0;
  [Pref="layout.animations.test.enabled"]
  undefined advanceClock(long millis);

  [Pref="dom.testbinding.prefcontrolled2.enabled"]
  readonly attribute boolean prefControlledAttributeEnabled;
  [Pref="dom.testbinding.prefcontrolled2.enabled"]
  static readonly attribute boolean prefControlledStaticAttributeEnabled;
  [Pref="dom.testbinding.prefcontrolled2.enabled"]
  undefined prefControlledMethodEnabled();
  [Pref="dom.testbinding.prefcontrolled2.enabled"]
  static undefined prefControlledStaticMethodEnabled();
  [Pref="dom.testbinding.prefcontrolled2.enabled"]
  const unsigned short prefControlledConstEnabled = 0;

  [Func="TestBinding::condition_unsatisfied"]
  readonly attribute boolean funcControlledAttributeDisabled;
  [Func="TestBinding::condition_unsatisfied"]
  static readonly attribute boolean funcControlledStaticAttributeDisabled;
  [Func="TestBinding::condition_unsatisfied"]
  undefined funcControlledMethodDisabled();
  [Func="TestBinding::condition_unsatisfied"]
  static undefined funcControlledStaticMethodDisabled();
  [Func="TestBinding::condition_unsatisfied"]
  const unsigned short funcControlledConstDisabled = 0;

  [Func="TestBinding::condition_satisfied"]
  readonly attribute boolean funcControlledAttributeEnabled;
  [Func="TestBinding::condition_satisfied"]
  static readonly attribute boolean funcControlledStaticAttributeEnabled;
  [Func="TestBinding::condition_satisfied"]
  undefined funcControlledMethodEnabled();
  [Func="TestBinding::condition_satisfied"]
  static undefined funcControlledStaticMethodEnabled();
  [Func="TestBinding::condition_satisfied"]
  const unsigned short funcControlledConstEnabled = 0;

  [Throws]
  Promise<any> returnResolvedPromise(any value);
  [Throws]
  Promise<any> returnRejectedPromise(any value);
  readonly attribute Promise<boolean> promiseAttribute;
  undefined acceptPromise(Promise<DOMString> string);
  Promise<any> promiseNativeHandler(SimpleCallback? resolve, SimpleCallback? reject);
  undefined promiseResolveNative(Promise<any> p, any value);
  undefined promiseRejectNative(Promise<any> p, any value);
  undefined promiseRejectWithTypeError(Promise<any> p, USVString message);
  undefined resolvePromiseDelayed(Promise<any> p, DOMString value, unsigned long long ms);

  undefined panic();

  GlobalScope entryGlobal();
  GlobalScope incumbentGlobal();

  [Exposed=(Window)]
  readonly attribute boolean semiExposedBoolFromInterface;

  TestDictionaryWithParent getDictionaryWithParent(DOMString parent, DOMString child);
};

[Exposed=(Window)]
partial interface TestBinding {
  readonly attribute boolean boolFromSemiExposedPartialInterface;
};

partial interface TestBinding {
  [Exposed=(Window)]
  readonly attribute boolean semiExposedBoolFromPartialInterface;
};

callback SimpleCallback = undefined(any value);
callback callbackWithOnlyOneOptionalArg = Promise<undefined> (optional any reason);

partial interface TestBinding {
  [Pref="dom.testable_crash.enabled"]
  undefined crashHard();
};

[Exposed=(Window,Worker), Pref="dom.testbinding.enabled"]
namespace TestNS {
    const unsigned long ONE   = 1;
    const unsigned long TWO   = 0x2;
};
