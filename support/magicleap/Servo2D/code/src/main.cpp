/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <Servo2D.h>
#include <ml_logging.h>

int main(int argc, char **argv)
{
  ML_LOG(Debug, "Servo2D Starting.");
  Servo2D myApp;
  return myApp.run();
}
