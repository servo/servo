#include "ClientRectBinding.h"
#include "ClientRectListBinding.h"
#include "DOMParserBinding.h"
#include "EventBinding.h"
#include "EventTargetBinding.h"
#include "HTMLCollectionBinding.h"
#include "nsScriptNameSpaceManager.h"

namespace mozilla {
namespace dom {
void
Register(nsScriptNameSpaceManager* aNameSpaceManager)
{

#define REGISTER_PROTO(_dom_class, _pref_check) \
  aNameSpaceManager->RegisterDefineDOMInterface(NS_LITERAL_STRING(#_dom_class), _dom_class##Binding::DefineDOMInterface, _pref_check);

REGISTER_PROTO(ClientRect, nullptr);
REGISTER_PROTO(ClientRectList, nullptr);
REGISTER_PROTO(DOMParser, nullptr);
REGISTER_PROTO(Event, nullptr);
REGISTER_PROTO(EventTarget, nullptr);
REGISTER_PROTO(HTMLCollection, nullptr);

#undef REGISTER_PROTO
}

} // namespace dom
} // namespace mozilla

