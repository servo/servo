/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include <mach/mach.h>

int macos_count_running_threads() {
  task_t task = current_task();
  thread_act_array_t threads;
  mach_msg_type_number_t tcnt;
  const kern_return_t status = task_threads(task, &threads, &tcnt);
  if (status == KERN_SUCCESS) {
    // Free data structures attached to the thread list.
    for (uint32_t t = 0; t < tcnt; t++) {
      mach_port_deallocate(task, threads[t]);
    }
    vm_deallocate(task, (vm_address_t)threads, sizeof(thread_t) * tcnt);
  }
  return tcnt;
}
