# Copyright (C) 2010 The Android Open Source Project
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
LOCAL_PATH := $(call my-dir)

include $(CLEAR_VARS)

LOCAL_MODULE := freeglut
LOCAL_SRC_FILES := libfreeglut-gles2.a

include $(PREBUILT_STATIC_LIBRARY)


include $(CLEAR_VARS)

LOCAL_MODULE := ServoAndroid

LOCAL_SRC_FILES := common.cpp android-dl.cpp main.cpp 

LOCAL_C_INCLUDES := $(LOCAL_PATH) \
	$(LOCAL_PATH)/../include

LOCAL_CXXFLAGS := -DFREEGLUT_GLES2 -gstabs+

LOCAL_LDLIBS := -ldl -llog -landroid -lGLESv2 -lGLESv1_CM -lEGL

LOCAL_STATIC_LIBRARIES := android_native_app_glue freeglut
LOCAL_SHARED_LIBRARIES := libdl

include $(BUILD_SHARED_LIBRARY)

$(call import-module,android/native_app_glue)
