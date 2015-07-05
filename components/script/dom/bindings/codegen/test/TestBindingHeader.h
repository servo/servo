/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef TestBindingHeader_h
#define TestBindingHeader_h

#include "nsWrapperCache.h"
#include "mozilla/ErrorResult.h"
#include "mozilla/dom/BindingUtils.h"
#include "mozilla/dom/TypedArray.h"
#include "nsCOMPtr.h"
// We don't export TestCodeGenBinding.h, but it's right in our parent dir.
#include "../TestCodeGenBinding.h"
#include "mozilla/dom/UnionTypes.h"

namespace mozilla {
namespace dom {

// IID for the TestNonCastableInterface
#define NS_TEST_NONCASTABLE_INTERFACE_IID \
{ 0x7c9f8ee2, 0xc9bf, 0x46ca, \
 { 0xa0, 0xa9, 0x03, 0xa8, 0xd6, 0x30, 0x0e, 0xde } }

class TestNonCastableInterface : public nsISupports,
                                 public nsWrapperCache
{
public:
  NS_DECLARE_STATIC_IID_ACCESSOR(NS_TEST_NONCASTABLE_INTERFACE_IID)
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();
};

// IID for the IndirectlyImplementedInterface
#define NS_INDIRECTLY_IMPLEMENTED_INTERFACE_IID \
{ 0xfed55b69, 0x7012, 0x4849, \
 { 0xaf, 0x56, 0x4b, 0xa9, 0xee, 0x41, 0x30, 0x89 } }

class IndirectlyImplementedInterface : public nsISupports,
                                       public nsWrapperCache
{
public:
  NS_DECLARE_STATIC_IID_ACCESSOR(NS_INDIRECTLY_IMPLEMENTED_INTERFACE_IID)
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  bool IndirectlyImplementedProperty();
  void IndirectlyImplementedProperty(bool);
  void IndirectlyImplementedMethod();
};

// IID for the TestExternalInterface
#define NS_TEST_EXTERNAL_INTERFACE_IID \
{ 0xd5ba0c99, 0x9b1d, 0x4e71, \
 { 0x8a, 0x94, 0x56, 0x38, 0x6c, 0xa3, 0xda, 0x3d } }
class TestExternalInterface : public nsISupports
{
public:
  NS_DECLARE_STATIC_IID_ACCESSOR(NS_TEST_EXTERNAL_INTERFACE_IID)
  NS_DECL_ISUPPORTS
};

// IID for the TestCallbackInterface
#define NS_TEST_CALLBACK_INTERFACE_IID \
{ 0xbf711ba4, 0xc8f6, 0x46cf, \
 { 0xba, 0x5b, 0xaa, 0xe2, 0x78, 0x18, 0xe6, 0x4a } }
class TestCallbackInterface : public nsISupports
{
public:
  NS_DECLARE_STATIC_IID_ACCESSOR(NS_TEST_CALLBACK_INTERFACE_IID)
  NS_DECL_ISUPPORTS
};

class TestNonWrapperCacheInterface : public nsISupports
{
public:
  NS_DECL_ISUPPORTS

  virtual JSObject* WrapObject(JSContext* cx, JSObject* scope);
};

class OnlyForUseInConstructor : public nsISupports,
                                public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS
  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();
};

class TestInterface : public nsISupports,
                      public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  // And now our actual WebIDL API
  // Constructors
  static
  already_AddRefed<TestInterface> Constructor(nsISupports*, ErrorResult&);
  static
  already_AddRefed<TestInterface> Constructor(nsISupports*, const nsAString&,
                                              ErrorResult&);
  static
  already_AddRefed<TestInterface> Constructor(nsISupports*, uint32_t,
                                              Nullable<bool>&, ErrorResult&);
  static
  already_AddRefed<TestInterface> Constructor(nsISupports*, TestInterface*,
                                              ErrorResult&);
  static
  already_AddRefed<TestInterface> Constructor(nsISupports*,
                                              TestNonCastableInterface&,
                                              ErrorResult&);
  /*  static
  already_AddRefed<TestInterface> Constructor(nsISupports*,
                                              uint32_t, uint32_t,
                                              const TestInterfaceOrOnlyForUseInConstructor&,
                                              ErrorResult&);
  */

  // Integer types
  int8_t ReadonlyByte();
  int8_t WritableByte();
  void SetWritableByte(int8_t);
  void PassByte(int8_t);
  int8_t ReceiveByte();
  void PassOptionalByte(const Optional<int8_t>&);
  void PassOptionalByteWithDefault(int8_t);
  void PassNullableByte(Nullable<int8_t>&);
  void PassOptionalNullableByte(const Optional< Nullable<int8_t> >&);

  int16_t ReadonlyShort();
  int16_t WritableShort();
  void SetWritableShort(int16_t);
  void PassShort(int16_t);
  int16_t ReceiveShort();
  void PassOptionalShort(const Optional<int16_t>&);
  void PassOptionalShortWithDefault(int16_t);

  int32_t ReadonlyLong();
  int32_t WritableLong();
  void SetWritableLong(int32_t);
  void PassLong(int32_t);
  int16_t ReceiveLong();
  void PassOptionalLong(const Optional<int32_t>&);
  void PassOptionalLongWithDefault(int32_t);

  int64_t ReadonlyLongLong();
  int64_t WritableLongLong();
  void SetWritableLongLong(int64_t);
  void PassLongLong(int64_t);
  int64_t ReceiveLongLong();
  void PassOptionalLongLong(const Optional<int64_t>&);
  void PassOptionalLongLongWithDefault(int64_t);

  uint8_t ReadonlyOctet();
  uint8_t WritableOctet();
  void SetWritableOctet(uint8_t);
  void PassOctet(uint8_t);
  uint8_t ReceiveOctet();
  void PassOptionalOctet(const Optional<uint8_t>&);
  void PassOptionalOctetWithDefault(uint8_t);

  uint16_t ReadonlyUnsignedShort();
  uint16_t WritableUnsignedShort();
  void SetWritableUnsignedShort(uint16_t);
  void PassUnsignedShort(uint16_t);
  uint16_t ReceiveUnsignedShort();
  void PassOptionalUnsignedShort(const Optional<uint16_t>&);
  void PassOptionalUnsignedShortWithDefault(uint16_t);

  uint32_t ReadonlyUnsignedLong();
  uint32_t WritableUnsignedLong();
  void SetWritableUnsignedLong(uint32_t);
  void PassUnsignedLong(uint32_t);
  uint32_t ReceiveUnsignedLong();
  void PassOptionalUnsignedLong(const Optional<uint32_t>&);
  void PassOptionalUnsignedLongWithDefault(uint32_t);

  uint64_t ReadonlyUnsignedLongLong();
  uint64_t WritableUnsignedLongLong();
  void SetWritableUnsignedLongLong(uint64_t);
  void PassUnsignedLongLong(uint64_t);
  uint64_t ReceiveUnsignedLongLong();
  void PassOptionalUnsignedLongLong(const Optional<uint64_t>&);
  void PassOptionalUnsignedLongLongWithDefault(uint64_t);

  // Interface types
  already_AddRefed<TestInterface> ReceiveSelf();
  already_AddRefed<TestInterface> ReceiveNullableSelf();
  TestInterface* ReceiveWeakSelf();
  TestInterface* ReceiveWeakNullableSelf();
  void PassSelf(TestInterface&);
  void PassSelf2(NonNull<TestInterface>&);
  void PassNullableSelf(TestInterface*);
  already_AddRefed<TestInterface> NonNullSelf();
  void SetNonNullSelf(TestInterface&);
  already_AddRefed<TestInterface> GetNullableSelf();
  void SetNullableSelf(TestInterface*);
  void PassOptionalSelf(const Optional<TestInterface*> &);
  void PassOptionalNonNullSelf(const Optional<NonNull<TestInterface> >&);
  void PassOptionalSelfWithDefault(TestInterface*);

  already_AddRefed<TestNonWrapperCacheInterface> ReceiveNonWrapperCacheInterface();
  already_AddRefed<TestNonWrapperCacheInterface> ReceiveNullableNonWrapperCacheInterface();
  void ReceiveNonWrapperCacheInterfaceSequence(nsTArray<nsRefPtr<TestNonWrapperCacheInterface> >&);
  void ReceiveNullableNonWrapperCacheInterfaceSequence(nsTArray<nsRefPtr<TestNonWrapperCacheInterface> >&);
  void ReceiveNonWrapperCacheInterfaceNullableSequence(Nullable<nsTArray<nsRefPtr<TestNonWrapperCacheInterface> > >&);
  void ReceiveNullableNonWrapperCacheInterfaceNullableSequence(
    Nullable<nsTArray<nsRefPtr<TestNonWrapperCacheInterface> > >&);

  already_AddRefed<TestNonCastableInterface> ReceiveOther();
  already_AddRefed<TestNonCastableInterface> ReceiveNullableOther();
  TestNonCastableInterface* ReceiveWeakOther();
  TestNonCastableInterface* ReceiveWeakNullableOther();
  void PassOther(TestNonCastableInterface&);
  void PassOther2(NonNull<TestNonCastableInterface>&);
  void PassNullableOther(TestNonCastableInterface*);
  already_AddRefed<TestNonCastableInterface> NonNullOther();
  void SetNonNullOther(TestNonCastableInterface&);
  already_AddRefed<TestNonCastableInterface> GetNullableOther();
  void SetNullableOther(TestNonCastableInterface*);
  void PassOptionalOther(const Optional<TestNonCastableInterface*>&);
  void PassOptionalNonNullOther(const Optional<NonNull<TestNonCastableInterface> >&);
  void PassOptionalOtherWithDefault(TestNonCastableInterface*);

  already_AddRefed<TestExternalInterface> ReceiveExternal();
  already_AddRefed<TestExternalInterface> ReceiveNullableExternal();
  TestExternalInterface* ReceiveWeakExternal();
  TestExternalInterface* ReceiveWeakNullableExternal();
  void PassExternal(TestExternalInterface*);
  void PassExternal2(TestExternalInterface*);
  void PassNullableExternal(TestExternalInterface*);
  already_AddRefed<TestExternalInterface> NonNullExternal();
  void SetNonNullExternal(TestExternalInterface*);
  already_AddRefed<TestExternalInterface> GetNullableExternal();
  void SetNullableExternal(TestExternalInterface*);
  void PassOptionalExternal(const Optional<TestExternalInterface*>&);
  void PassOptionalNonNullExternal(const Optional<TestExternalInterface*>&);
  void PassOptionalExternalWithDefault(TestExternalInterface*);

  already_AddRefed<TestCallbackInterface> ReceiveCallbackInterface();
  already_AddRefed<TestCallbackInterface> ReceiveNullableCallbackInterface();
  TestCallbackInterface* ReceiveWeakCallbackInterface();
  TestCallbackInterface* ReceiveWeakNullableCallbackInterface();
  void PassCallbackInterface(TestCallbackInterface&);
  void PassCallbackInterface2(OwningNonNull<TestCallbackInterface>);
  void PassNullableCallbackInterface(TestCallbackInterface*);
  already_AddRefed<TestCallbackInterface> NonNullCallbackInterface();
  void SetNonNullCallbackInterface(TestCallbackInterface&);
  already_AddRefed<TestCallbackInterface> GetNullableCallbackInterface();
  void SetNullableCallbackInterface(TestCallbackInterface*);
  void PassOptionalCallbackInterface(const Optional<nsRefPtr<TestCallbackInterface> >&);
  void PassOptionalNonNullCallbackInterface(const Optional<OwningNonNull<TestCallbackInterface> >&);
  void PassOptionalCallbackInterfaceWithDefault(TestCallbackInterface*);

  already_AddRefed<IndirectlyImplementedInterface> ReceiveConsequentialInterface();
  void PassConsequentialInterface(IndirectlyImplementedInterface&);

  // Sequence types
  void ReceiveSequence(nsTArray<int32_t>&);
  void ReceiveNullableSequence(Nullable< nsTArray<int32_t> >&);
  void ReceiveSequenceOfNullableInts(nsTArray< Nullable<int32_t> >&);
  void ReceiveNullableSequenceOfNullableInts(Nullable< nsTArray< Nullable<int32_t> > >&);
  void PassSequence(const Sequence<int32_t> &);
  void PassNullableSequence(const Nullable< Sequence<int32_t> >&);
  void PassSequenceOfNullableInts(const Sequence<Nullable<int32_t> >&);
  void PassOptionalSequenceOfNullableInts(const Optional<Sequence<Nullable<int32_t> > > &);
  void PassOptionalNullableSequenceOfNullableInts(const Optional<Nullable<Sequence<Nullable<int32_t> > > > &);
  void ReceiveCastableObjectSequence(nsTArray< nsRefPtr<TestInterface> > &);
  void ReceiveNullableCastableObjectSequence(nsTArray< nsRefPtr<TestInterface> > &);
  void ReceiveCastableObjectNullableSequence(Nullable< nsTArray< nsRefPtr<TestInterface> > >&);
  void ReceiveNullableCastableObjectNullableSequence(Nullable< nsTArray< nsRefPtr<TestInterface> > >&);
  void ReceiveWeakCastableObjectSequence(nsTArray<TestInterface*> &);
  void ReceiveWeakNullableCastableObjectSequence(nsTArray<TestInterface*> &);
  void ReceiveWeakCastableObjectNullableSequence(Nullable< nsTArray<TestInterface*> >&);
  void ReceiveWeakNullableCastableObjectNullableSequence(Nullable< nsTArray<TestInterface*> >&);
  void PassCastableObjectSequence(const Sequence< OwningNonNull<TestInterface> >&);
  void PassNullableCastableObjectSequence(const Sequence< nsRefPtr<TestInterface> > &);
  void PassCastableObjectNullableSequence(const Nullable< Sequence< OwningNonNull<TestInterface> > >&);
  void PassNullableCastableObjectNullableSequence(const Nullable< Sequence< nsRefPtr<TestInterface> > >&);
  void PassOptionalSequence(const Optional<Sequence<int32_t> >&);
  void PassOptionalNullableSequence(const Optional<Nullable<Sequence<int32_t> > >&);
  void PassOptionalNullableSequenceWithDefaultValue(const Nullable< Sequence<int32_t> >&);
  void PassOptionalObjectSequence(const Optional<Sequence<OwningNonNull<TestInterface> > >&);

  void ReceiveStringSequence(nsTArray<nsString>&);
  void PassStringSequence(const Sequence<nsString>&);

  void ReceiveAnySequence(JSContext*, nsTArray<JS::Value>&);
  void ReceiveNullableAnySequence(JSContext*, Nullable<nsTArray<JS::Value> >);

  // Typed array types
  void PassArrayBuffer(ArrayBuffer&);
  void PassNullableArrayBuffer(ArrayBuffer*);
  void PassOptionalArrayBuffer(const Optional<ArrayBuffer>&);
  void PassOptionalNullableArrayBuffer(const Optional<ArrayBuffer*>&);
  void PassOptionalNullableArrayBufferWithDefaultValue(ArrayBuffer*);
  void PassArrayBufferView(ArrayBufferView&);
  void PassInt8Array(Int8Array&);
  void PassInt16Array(Int16Array&);
  void PassInt32Array(Int32Array&);
  void PassUint8Array(Uint8Array&);
  void PassUint16Array(Uint16Array&);
  void PassUint32Array(Uint32Array&);
  void PassUint8ClampedArray(Uint8ClampedArray&);
  void PassFloat32Array(Float32Array&);
  void PassFloat64Array(Float64Array&);
  JSObject* ReceiveUint8Array(JSContext*);

  // String types
  void PassString(const nsAString&);
  void PassNullableString(const nsAString&);
  void PassOptionalString(const Optional<nsAString>&);
  void PassOptionalStringWithDefaultValue(const nsAString&);
  void PassOptionalNullableString(const Optional<nsAString>&);
  void PassOptionalNullableStringWithDefaultValue(const nsAString&);

  // Enumarated types
  void PassEnum(TestEnum);
  void PassOptionalEnum(const Optional<TestEnum>&);
  void PassEnumWithDefault(TestEnum);
  TestEnum ReceiveEnum();
  TestEnum EnumAttribute();
  TestEnum ReadonlyEnumAttribute();
  void SetEnumAttribute(TestEnum);

  // Callback types
  void PassCallback(JSContext*, JSObject*);
  void PassNullableCallback(JSContext*, JSObject*);
  void PassOptionalCallback(JSContext*, const Optional<JSObject*>&);
  void PassOptionalNullableCallback(JSContext*, const Optional<JSObject*>&);
  void PassOptionalNullableCallbackWithDefaultValue(JSContext*, JSObject*);
  JSObject* ReceiveCallback(JSContext*);
  JSObject* ReceiveNullableCallback(JSContext*);

  // Any types
  void PassAny(JSContext*, JS::Value);
  void PassOptionalAny(JSContext*, const Optional<JS::Value>&);
  void PassAnyDefaultNull(JSContext*, JS::Value);
  JS::Value ReceiveAny(JSContext*);

  // object types
  void PassObject(JSContext*, JSObject&);
  void PassNullableObject(JSContext*, JSObject*);
  void PassOptionalObject(JSContext*, const Optional<NonNull<JSObject> >&);
  void PassOptionalNullableObject(JSContext*, const Optional<JSObject*>&);
  void PassOptionalNullableObjectWithDefaultValue(JSContext*, JSObject*);
  JSObject* ReceiveObject(JSContext*);
  JSObject* ReceiveNullableObject(JSContext*);

  // Union types
  void PassUnion(JSContext*, const ObjectOrLong& arg);
  void PassUnionWithNullable(JSContext*, const ObjectOrNullOrLong& arg)
  {
    ObjectOrLong returnValue;
    if (arg.IsNull()) {
    } else if (arg.IsObject()) {
      JSObject& obj = (JSObject&)arg.GetAsObject();
      JS_GetClass(&obj);
      //returnValue.SetAsObject(&obj);
    } else {
      int32_t i = arg.GetAsLong();
      i += 1;
    }
  }
  void PassNullableUnion(JSContext*, const Nullable<ObjectOrLong>&);
  void PassOptionalUnion(JSContext*, const Optional<ObjectOrLong>&);
  void PassOptionalNullableUnion(JSContext*, const Optional<Nullable<ObjectOrLong> >&);
  void PassOptionalNullableUnionWithDefaultValue(JSContext*, const Nullable<ObjectOrLong>&);
  //void PassUnionWithInterfaces(const TestInterfaceOrTestExternalInterface& arg);
  //void PassUnionWithInterfacesAndNullable(const TestInterfaceOrNullOrTestExternalInterface& arg);
  void PassUnionWithArrayBuffer(const ArrayBufferOrLong&);
  void PassUnionWithString(JSContext*, const StringOrObject&);
  //void PassUnionWithEnum(JSContext*, const TestEnumOrObject&);
  void PassUnionWithCallback(JSContext*, const TestCallbackOrLong&);
  void PassUnionWithObject(JSContext*, const ObjectOrLong&);

  // binaryNames tests
  void MethodRenamedTo();
  void MethodRenamedTo(int8_t);
  int8_t AttributeGetterRenamedTo();
  int8_t AttributeRenamedTo();
  void SetAttributeRenamedTo(int8_t);

  // Dictionary tests
  void PassDictionary(const Dict&);
  void PassOtherDictionary(const GrandparentDict&);
  void PassSequenceOfDictionaries(const Sequence<Dict>&);
  void PassDictionaryOrLong(const Dict&);
  void PassDictionaryOrLong(int32_t);
  void PassDictContainingDict(const DictContainingDict&);
  void PassDictContainingSequence(const DictContainingSequence&);

  // Typedefs
  void ExerciseTypedefInterfaces1(TestInterface&);
  already_AddRefed<TestInterface> ExerciseTypedefInterfaces2(TestInterface*);
  void ExerciseTypedefInterfaces3(TestInterface&);

  // Miscellania
  int32_t AttrWithLenientThis();
  void SetAttrWithLenientThis(int32_t);

  // Methods and properties imported via "implements"
  bool ImplementedProperty();
  void SetImplementedProperty(bool);
  void ImplementedMethod();
  bool ImplementedParentProperty();
  void SetImplementedParentProperty(bool);
  void ImplementedParentMethod();
  bool IndirectlyImplementedProperty();
  void SetIndirectlyImplementedProperty(bool);
  void IndirectlyImplementedMethod();
  uint32_t DiamondImplementedProperty();

  // Test EnforceRange/Clamp
  void DontEnforceRangeOrClamp(int8_t);
  void DoEnforceRange(int8_t);
  void DoClamp(int8_t);

private:
  // We add signatures here that _could_ start matching if the codegen
  // got data types wrong.  That way if it ever does we'll have a call
  // to these private deleted methods and compilation will fail.
  void SetReadonlyByte(int8_t) MOZ_DELETE;
  template<typename T>
  void SetWritableByte(T) MOZ_DELETE;
  template<typename T>
  void PassByte(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalByte(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalByteWithDefault(T) MOZ_DELETE;

  void SetReadonlyShort(int16_t) MOZ_DELETE;
  template<typename T>
  void SetWritableShort(T) MOZ_DELETE;
  template<typename T>
  void PassShort(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalShort(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalShortWithDefault(T) MOZ_DELETE;

  void SetReadonlyLong(int32_t) MOZ_DELETE;
  template<typename T>
  void SetWritableLong(T) MOZ_DELETE;
  template<typename T>
  void PassLong(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalLong(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalLongWithDefault(T) MOZ_DELETE;

  void SetReadonlyLongLong(int64_t) MOZ_DELETE;
  template<typename T>
  void SetWritableLongLong(T) MOZ_DELETE;
  template<typename T>
  void PassLongLong(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalLongLong(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalLongLongWithDefault(T) MOZ_DELETE;

  void SetReadonlyOctet(uint8_t) MOZ_DELETE;
  template<typename T>
  void SetWritableOctet(T) MOZ_DELETE;
  template<typename T>
  void PassOctet(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalOctet(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalOctetWithDefault(T) MOZ_DELETE;

  void SetReadonlyUnsignedShort(uint16_t) MOZ_DELETE;
  template<typename T>
  void SetWritableUnsignedShort(T) MOZ_DELETE;
  template<typename T>
  void PassUnsignedShort(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalUnsignedShort(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalUnsignedShortWithDefault(T) MOZ_DELETE;

  void SetReadonlyUnsignedLong(uint32_t) MOZ_DELETE;
  template<typename T>
  void SetWritableUnsignedLong(T) MOZ_DELETE;
  template<typename T>
  void PassUnsignedLong(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalUnsignedLong(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalUnsignedLongWithDefault(T) MOZ_DELETE;

  void SetReadonlyUnsignedLongLong(uint64_t) MOZ_DELETE;
  template<typename T>
  void SetWritableUnsignedLongLong(T) MOZ_DELETE;
  template<typename T>
  void PassUnsignedLongLong(T) MOZ_DELETE;
  template<typename T>
  void PassOptionalUnsignedLongLong(const Optional<T>&) MOZ_DELETE;
  template<typename T>
  void PassOptionalUnsignedLongLongWithDefault(T) MOZ_DELETE;

  // Enforce that only const things are passed for sequences
  void PassSequence(Sequence<int32_t> &) MOZ_DELETE;
  void PassNullableSequence(Nullable< Sequence<int32_t> >&) MOZ_DELETE;
  void PassOptionalNullableSequenceWithDefaultValue(Nullable< Sequence<int32_t> >&) MOZ_DELETE;

  // Enforce that only const things are passed for optional
  void PassOptionalByte(Optional<int8_t>&) MOZ_DELETE;
  void PassOptionalNullableByte(Optional<Nullable<int8_t> >&) MOZ_DELETE;
  void PassOptionalShort(Optional<int16_t>&) MOZ_DELETE;
  void PassOptionalLong(Optional<int32_t>&) MOZ_DELETE;
  void PassOptionalLongLong(Optional<int64_t>&) MOZ_DELETE;
  void PassOptionalOctet(Optional<uint8_t>&) MOZ_DELETE;
  void PassOptionalUnsignedShort(Optional<uint16_t>&) MOZ_DELETE;
  void PassOptionalUnsignedLong(Optional<uint32_t>&) MOZ_DELETE;
  void PassOptionalUnsignedLongLong(Optional<uint64_t>&) MOZ_DELETE;
  void PassOptionalSelf(Optional<TestInterface*> &) MOZ_DELETE;
  void PassOptionalNonNullSelf(Optional<NonNull<TestInterface> >&) MOZ_DELETE;
  void PassOptionalOther(Optional<TestNonCastableInterface*>&);
  void PassOptionalNonNullOther(Optional<NonNull<TestNonCastableInterface> >&);
  void PassOptionalExternal(Optional<TestExternalInterface*>&) MOZ_DELETE;
  void PassOptionalNonNullExternal(Optional<TestExternalInterface*>&) MOZ_DELETE;
  void PassOptionalSequence(Optional<Sequence<int32_t> >&) MOZ_DELETE;
  void PassOptionalNullableSequence(Optional<Nullable<Sequence<int32_t> > >&) MOZ_DELETE;
  void PassOptionalObjectSequence(Optional<Sequence<OwningNonNull<TestInterface> > >&) MOZ_DELETE;
  void PassOptionalArrayBuffer(Optional<ArrayBuffer>&) MOZ_DELETE;
  void PassOptionalNullableArrayBuffer(Optional<ArrayBuffer*>&) MOZ_DELETE;
  void PassOptionalEnum(Optional<TestEnum>&) MOZ_DELETE;
  void PassOptionalCallback(JSContext*, Optional<JSObject*>&) MOZ_DELETE;
  void PassOptionalNullableCallback(JSContext*, Optional<JSObject*>&) MOZ_DELETE;
  void PassOptionalAny(Optional<JS::Value>&) MOZ_DELETE;

  // And test that string stuff is always const
  void PassString(nsAString&) MOZ_DELETE;
  void PassNullableString(nsAString&) MOZ_DELETE;
  void PassOptionalString(Optional<nsAString>&) MOZ_DELETE;
  void PassOptionalStringWithDefaultValue(nsAString&) MOZ_DELETE;
  void PassOptionalNullableString(Optional<nsAString>&) MOZ_DELETE;
  void PassOptionalNullableStringWithDefaultValue(nsAString&) MOZ_DELETE;

};

class TestIndexedGetterInterface : public nsISupports,
                                   public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  uint32_t IndexedGetter(uint32_t, bool&);
  uint32_t IndexedGetter(uint32_t&) MOZ_DELETE;
  uint32_t Item(uint32_t&);
  uint32_t Item(uint32_t, bool&) MOZ_DELETE;
  uint32_t Length();
};

class TestNamedGetterInterface : public nsISupports,
                                 public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  void NamedGetter(const nsAString&, bool&, nsAString&);
};

class TestIndexedAndNamedGetterInterface : public nsISupports,
                                           public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  uint32_t IndexedGetter(uint32_t, bool&);
  void NamedGetter(const nsAString&, bool&, nsAString&);
  void NamedItem(const nsAString&, nsAString&);
  uint32_t Length();
};

class TestIndexedSetterInterface : public nsISupports,
                                   public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  void IndexedSetter(uint32_t, const nsAString&);
  void SetItem(uint32_t, const nsAString&);
};

class TestNamedSetterInterface : public nsISupports,
                                 public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  void NamedSetter(const nsAString&, TestIndexedSetterInterface&);
};

class TestIndexedAndNamedSetterInterface : public nsISupports,
                                           public nsWrapperCache
{
public:
  NS_DECL_ISUPPORTS

  // We need a GetParentObject to make binding codegen happy
  virtual nsISupports* GetParentObject();

  void IndexedSetter(uint32_t, TestIndexedSetterInterface&);
  void NamedSetter(const nsAString&, TestIndexedSetterInterface&);
  void SetNamedItem(const nsAString&, TestIndexedSetterInterface&);
};

class TestIndexedAndNamedGetterAndSetterInterface : public TestIndexedSetterInterface
{
public:
  uint32_t IndexedGetter(uint32_t, bool&);
  uint32_t Item(uint32_t);
  void NamedGetter(const nsAString&, bool&, nsAString&);
  void NamedItem(const nsAString&, nsAString&);
  void IndexedSetter(uint32_t, int32_t&);
  void IndexedSetter(uint32_t, const nsAString&) MOZ_DELETE;
  void NamedSetter(const nsAString&, const nsAString&);
  void Stringify(nsAString&);
  uint32_t Length();
};

} // namespace dom
} // namespace mozilla

#endif /* TestBindingHeader_h */
