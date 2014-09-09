/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim:set ts=2 sw=2 sts=2 et cindent: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "Skeleton.h"
#include "mozilla/dom/SkeletonBinding.h"
#include "nsContentUtils.h"

namespace mozilla {
namespace dom {

NS_IMPL_CYCLE_COLLECTION_WRAPPERCACHE_0(Skeleton)
NS_IMPL_CYCLE_COLLECTING_ADDREF(Skeleton)
NS_IMPL_CYCLE_COLLECTING_RELEASE(Skeleton)
NS_INTERFACE_MAP_BEGIN_CYCLE_COLLECTION(Skeleton)
  NS_WRAPPERCACHE_INTERFACE_MAP_ENTRY
  NS_INTERFACE_MAP_ENTRY(nsISupports)
NS_INTERFACE_MAP_END

Skeleton::Skeleton()
{
  SetIsDOMBinding();
}

Skeleton::~Skeleton()
{
}

JSObject*
Skeleton::WrapObject(JSContext* aCx, JSObject* aScope,
                         bool* aTriedToWrap)
{
  return SkeletonBinding::Wrap(aCx, aScope, this, aTriedToWrap);
}

}
}

