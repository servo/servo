/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include <Servo2D.h>
#include <ml_lifecycle.h>
#include <ml_logging.h>

int main(int argc, char **argv)
{
  ML_LOG(Debug, "Servo2D Starting.");

  // Handle optional initialization string passed via 'mldb launch'
  MLLifecycleInitArgList* list = NULL;
  MLLifecycleGetInitArgList(&list);
  const char* uri = NULL;
  if (nullptr != list) {
    int64_t list_length = 0;
    MLLifecycleGetInitArgListLength(list, &list_length);
    if (list_length > 0) {
      const MLLifecycleInitArg* iarg = NULL;
      MLLifecycleGetInitArgByIndex(list, 0, &iarg);
      if (nullptr != iarg) {
        MLLifecycleGetInitArgUri(iarg, &uri);
      }
    }
  }

  const char* args = getenv("SERVO_ARGS");

  Servo2D myApp(uri, args);
  int rv = myApp.run();

  MLLifecycleFreeInitArgList(&list);
  return rv;
}
