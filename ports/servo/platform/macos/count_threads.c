/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include <mach/mach.h>

int macos_count_running_threads() {
  task_t task = current_task();
  thread_act_array_t threads;
  mach_msg_type_number_t tcnt;
  task_threads(task, &threads, &tcnt);
  return tcnt;
}
