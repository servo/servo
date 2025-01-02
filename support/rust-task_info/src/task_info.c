// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include <mach/mach_init.h>
#include <mach/task.h>

static int
TaskBasicInfo(struct task_basic_info* info)
{
    mach_msg_type_number_t count = TASK_BASIC_INFO_COUNT;
    kern_return_t kr = task_info(mach_task_self(), TASK_BASIC_INFO,
                                 (task_info_t)info, &count);
    return kr == KERN_SUCCESS ? 0 : -1;
}

int
TaskBasicInfoVirtualSize(size_t* virtualSize)
{
    struct task_basic_info ti;
    int rv = TaskBasicInfo(&ti);
    *virtualSize = (rv == 0) ? ti.virtual_size : 0;
    return rv;
}

int
TaskBasicInfoResidentSize(size_t* residentSize)
{
    struct task_basic_info ti;
    int rv = TaskBasicInfo(&ti);
    *residentSize = (rv == 0) ? ti.resident_size : 0;
    return rv;
}

