/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*-*/
/* vim: set ts=2 sw=2 et tw=79: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_dom_Nullable_h
#define mozilla_dom_Nullable_h

#include "mozilla/Assertions.h"

namespace mozilla {
namespace dom {

// Support for nullable types
template <typename T>
struct Nullable
{
private:
  T mValue;
  bool mIsNull;

public:
  Nullable()
    : mIsNull(true)
  {}

  Nullable(T aValue)
    : mValue(aValue)
    , mIsNull(false)
  {}

  void SetValue(T aValue) {
    mValue = aValue;
    mIsNull = false;
  }

  // For cases when |T| is some type with nontrivial copy behavior, we may want
  // to get a reference to our internal copy of T and work with it directly
  // instead of relying on the copying version of SetValue().
  T& SetValue() {
    mIsNull = false;
    return mValue;
  }

  void SetNull() {
    mIsNull = true;
  }

  const T& Value() const {
    MOZ_ASSERT(!mIsNull);
    return mValue;
  }

  T& Value() {
    MOZ_ASSERT(!mIsNull);
    return mValue;
  }

  bool IsNull() const {
    return mIsNull;
  }
};

} // namespace dom
} // namespace mozilla

#endif /* mozilla_dom_Nullable_h */
