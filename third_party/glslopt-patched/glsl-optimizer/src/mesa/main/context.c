/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 1999-2007  Brian Paul   All Rights Reserved.
 * Copyright (C) 2008  VMware, Inc.  All Rights Reserved.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
 * OTHER DEALINGS IN THE SOFTWARE.
 */

/**
 * \file context.c
 * Mesa context/visual/framebuffer management functions.
 * \author Brian Paul
 */

/**
 * \mainpage Mesa Main Module
 *
 * \section MainIntroduction Introduction
 *
 * The Mesa Main module consists of all the files in the main/ directory.
 * Among the features of this module are:
 * <UL>
 * <LI> Structures to represent most GL state </LI>
 * <LI> State set/get functions </LI>
 * <LI> Display lists </LI>
 * <LI> Texture unit, object and image handling </LI>
 * <LI> Matrix and attribute stacks </LI>
 * </UL>
 *
 * Other modules are responsible for API dispatch, vertex transformation,
 * point/line/triangle setup, rasterization, vertex array caching,
 * vertex/fragment programs/shaders, etc.
 *
 *
 * \section AboutDoxygen About Doxygen
 *
 * If you're viewing this information as Doxygen-generated HTML you'll
 * see the documentation index at the top of this page.
 *
 * The first line lists the Mesa source code modules.
 * The second line lists the indexes available for viewing the documentation
 * for each module.
 *
 * Selecting the <b>Main page</b> link will display a summary of the module
 * (this page).
 *
 * Selecting <b>Data Structures</b> will list all C structures.
 *
 * Selecting the <b>File List</b> link will list all the source files in
 * the module.
 * Selecting a filename will show a list of all functions defined in that file.
 *
 * Selecting the <b>Data Fields</b> link will display a list of all
 * documented structure members.
 *
 * Selecting the <b>Globals</b> link will display a list
 * of all functions, structures, global variables and macros in the module.
 *
 */


#include "glheader.h"

#include "accum.h"
#include "api_exec.h"
#include "api_loopback.h"
#include "arrayobj.h"
#include "attrib.h"
#include "bbox.h"
#include "blend.h"
#include "buffers.h"
#include "bufferobj.h"
#include "conservativeraster.h"
#include "context.h"
#include "cpuinfo.h"
#include "debug.h"
#include "debug_output.h"
#include "depth.h"
#include "dlist.h"
#include "eval.h"
#include "extensions.h"
#include "fbobject.h"
#include "feedback.h"
#include "fog.h"
#include "formats.h"
#include "framebuffer.h"
#include "glthread.h"
#include "hint.h"
#include "hash.h"
#include "light.h"
#include "lines.h"
#include "macros.h"
#include "matrix.h"
#include "multisample.h"
#include "performance_monitor.h"
#include "performance_query.h"
#include "pipelineobj.h"
#include "pixel.h"
#include "pixelstore.h"
#include "points.h"
#include "polygon.h"
#include "queryobj.h"
#include "syncobj.h"
#include "rastpos.h"
#include "remap.h"
#include "scissor.h"
#include "shared.h"
#include "shaderobj.h"
#include "shaderimage.h"
#include "state.h"
#include "util/debug.h"
#include "util/disk_cache.h"
#include "util/strtod.h"
#include "stencil.h"
#include "shaderimage.h"
#include "texcompress_s3tc.h"
#include "texstate.h"
#include "transformfeedback.h"
#include "mtypes.h"
#include "varray.h"
#include "version.h"
#include "viewport.h"
#include "texturebindless.h"
#include "program/program.h"
#include "math/m_matrix.h"
#include "main/dispatch.h" /* for _gloffset_COUNT */
#include "macros.h"
#include "git_sha1.h"

#ifdef USE_SPARC_ASM
#include "sparc/sparc.h"
#endif

#include "compiler/glsl_types.h"
#include "compiler/glsl/builtin_functions.h"
#include "compiler/glsl/glsl_parser_extras.h"
#include <stdbool.h>
#include "util/u_memory.h"


#ifndef MESA_VERBOSE
int MESA_VERBOSE = 0;
#endif

#ifndef MESA_DEBUG_FLAGS
int MESA_DEBUG_FLAGS = 0;
#endif


/* ubyte -> float conversion */
GLfloat _mesa_ubyte_to_float_color_tab[256];



/**
 * Swap buffers notification callback.
 *
 * \param ctx GL context.
 *
 * Called by window system just before swapping buffers.
 * We have to finish any pending rendering.
 */
void
_mesa_notifySwapBuffers(struct gl_context *ctx)
{
   if (MESA_VERBOSE & VERBOSE_SWAPBUFFERS)
      _mesa_debug(ctx, "SwapBuffers\n");
   FLUSH_VERTICES(ctx, 0);
   if (ctx->Driver.Flush) {
      ctx->Driver.Flush(ctx);
   }
}


/**********************************************************************/
/** \name GL Visual allocation/destruction                            */
/**********************************************************************/
/*@{*/

/**
 * Allocates a struct gl_config structure and initializes it via
 * _mesa_initialize_visual().
 *
 * \param dbFlag double buffering
 * \param stereoFlag stereo buffer
 * \param depthBits requested bits per depth buffer value. Any value in [0, 32]
 * is acceptable but the actual depth type will be GLushort or GLuint as
 * needed.
 * \param stencilBits requested minimum bits per stencil buffer value
 * \param accumRedBits, accumGreenBits, accumBlueBits, accumAlphaBits number
 * of bits per color component in accum buffer.
 * \param redBits number of bits per color component in frame buffer for RGB(A)
 * mode.  We always use 8 in core Mesa though.
 * \param greenBits same as above.
 * \param blueBits same as above.
 * \param alphaBits same as above.
 * \param numSamples not really used.
 *
 * \return pointer to new struct gl_config or NULL if requested parameters
 * can't be met.
 *
 * \note Need to add params for level and numAuxBuffers (at least)
 */
struct gl_config *
_mesa_create_visual( GLboolean dbFlag,
                     GLboolean stereoFlag,
                     GLint redBits,
                     GLint greenBits,
                     GLint blueBits,
                     GLint alphaBits,
                     GLint depthBits,
                     GLint stencilBits,
                     GLint accumRedBits,
                     GLint accumGreenBits,
                     GLint accumBlueBits,
                     GLint accumAlphaBits,
                     GLuint numSamples )
{
   struct gl_config *vis = CALLOC_STRUCT(gl_config);
   if (vis) {
      if (!_mesa_initialize_visual(vis, dbFlag, stereoFlag,
                                   redBits, greenBits, blueBits, alphaBits,
                                   depthBits, stencilBits,
                                   accumRedBits, accumGreenBits,
                                   accumBlueBits, accumAlphaBits,
                                   numSamples)) {
         free(vis);
         return NULL;
      }
   }
   return vis;
}


/**
 * Makes some sanity checks and fills in the fields of the struct
 * gl_config object with the given parameters.  If the caller needs to
 * set additional fields, he should just probably init the whole
 * gl_config object himself.
 *
 * \return GL_TRUE on success, or GL_FALSE on failure.
 *
 * \sa _mesa_create_visual() above for the parameter description.
 */
GLboolean
_mesa_initialize_visual( struct gl_config *vis,
                         GLboolean dbFlag,
                         GLboolean stereoFlag,
                         GLint redBits,
                         GLint greenBits,
                         GLint blueBits,
                         GLint alphaBits,
                         GLint depthBits,
                         GLint stencilBits,
                         GLint accumRedBits,
                         GLint accumGreenBits,
                         GLint accumBlueBits,
                         GLint accumAlphaBits,
                         GLuint numSamples )
{
   assert(vis);

   if (depthBits < 0 || depthBits > 32) {
      return GL_FALSE;
   }
   if (stencilBits < 0 || stencilBits > 8) {
      return GL_FALSE;
   }
   assert(accumRedBits >= 0);
   assert(accumGreenBits >= 0);
   assert(accumBlueBits >= 0);
   assert(accumAlphaBits >= 0);

   vis->doubleBufferMode = dbFlag;
   vis->stereoMode       = stereoFlag;

   vis->redBits          = redBits;
   vis->greenBits        = greenBits;
   vis->blueBits         = blueBits;
   vis->alphaBits        = alphaBits;
   vis->rgbBits          = redBits + greenBits + blueBits;

   vis->depthBits      = depthBits;
   vis->stencilBits    = stencilBits;

   vis->accumRedBits   = accumRedBits;
   vis->accumGreenBits = accumGreenBits;
   vis->accumBlueBits  = accumBlueBits;
   vis->accumAlphaBits = accumAlphaBits;

   vis->numAuxBuffers = 0;
   vis->level = 0;
   vis->sampleBuffers = numSamples > 0 ? 1 : 0;
   vis->samples = numSamples;

   return GL_TRUE;
}


/**
 * Destroy a visual and free its memory.
 *
 * \param vis visual.
 *
 * Frees the visual structure.
 */
void
_mesa_destroy_visual( struct gl_config *vis )
{
   free(vis);
}

/*@}*/


/**********************************************************************/
/** \name Context allocation, initialization, destroying
 *
 * The purpose of the most initialization functions here is to provide the
 * default state values according to the OpenGL specification.
 */
/**********************************************************************/
/*@{*/


/**
 * One-time initialization mutex lock.
 *
 * \sa Used by one_time_init().
 */
mtx_t OneTimeLock = _MTX_INITIALIZER_NP;


/**
 * Calls all the various one-time-fini functions in Mesa
 */

static void
one_time_fini(void)
{
   glsl_type_singleton_decref();
   _mesa_locale_fini();
}

/**
 * Calls all the various one-time-init functions in Mesa.
 *
 * While holding a global mutex lock, calls several initialization functions,
 * and sets the glapi callbacks if the \c MESA_DEBUG environment variable is
 * defined.
 *
 * \sa _math_init().
 */
void
_mesa_initialize(void)
{
   static bool initialized;

   mtx_lock(&OneTimeLock);

   /* truly one-time init */
   if (!initialized) {
      GLuint i;

      STATIC_ASSERT(sizeof(GLbyte) == 1);
      STATIC_ASSERT(sizeof(GLubyte) == 1);
      STATIC_ASSERT(sizeof(GLshort) == 2);
      STATIC_ASSERT(sizeof(GLushort) == 2);
      STATIC_ASSERT(sizeof(GLint) == 4);
      STATIC_ASSERT(sizeof(GLuint) == 4);

      _mesa_locale_init();

      _mesa_one_time_init_extension_overrides();

      _mesa_get_cpu_features();

      for (i = 0; i < 256; i++) {
         _mesa_ubyte_to_float_color_tab[i] = (float) i / 255.0F;
      }

      atexit(one_time_fini);

#if defined(DEBUG)
      if (MESA_VERBOSE != 0) {
         _mesa_debug(NULL, "Mesa " PACKAGE_VERSION " DEBUG build" MESA_GIT_SHA1 "\n");
      }
#endif

      /* Take a glsl type reference for the duration of libGL's life to avoid
       * unecessary creation/destruction of glsl types.
       */
      glsl_type_singleton_init_or_ref();

      _mesa_init_remap_table();
   }

   initialized = true;

   mtx_unlock(&OneTimeLock);
}


/**
 * Initialize fields of gl_current_attrib (aka ctx->Current.*)
 */
static void
_mesa_init_current(struct gl_context *ctx)
{
   GLuint i;

   /* Init all to (0,0,0,1) */
   for (i = 0; i < ARRAY_SIZE(ctx->Current.Attrib); i++) {
      ASSIGN_4V( ctx->Current.Attrib[i], 0.0, 0.0, 0.0, 1.0 );
   }

   /* redo special cases: */
   ASSIGN_4V( ctx->Current.Attrib[VERT_ATTRIB_NORMAL], 0.0, 0.0, 1.0, 1.0 );
   ASSIGN_4V( ctx->Current.Attrib[VERT_ATTRIB_COLOR0], 1.0, 1.0, 1.0, 1.0 );
   ASSIGN_4V( ctx->Current.Attrib[VERT_ATTRIB_COLOR1], 0.0, 0.0, 0.0, 1.0 );
   ASSIGN_4V( ctx->Current.Attrib[VERT_ATTRIB_COLOR_INDEX], 1.0, 0.0, 0.0, 1.0 );
   ASSIGN_4V( ctx->Current.Attrib[VERT_ATTRIB_EDGEFLAG], 1.0, 0.0, 0.0, 1.0 );
}


/**
 * Init vertex/fragment/geometry program limits.
 * Important: drivers should override these with actual limits.
 */
static void
init_program_limits(struct gl_constants *consts, gl_shader_stage stage,
                    struct gl_program_constants *prog)
{
   prog->MaxInstructions = MAX_PROGRAM_INSTRUCTIONS;
   prog->MaxAluInstructions = MAX_PROGRAM_INSTRUCTIONS;
   prog->MaxTexInstructions = MAX_PROGRAM_INSTRUCTIONS;
   prog->MaxTexIndirections = MAX_PROGRAM_INSTRUCTIONS;
   prog->MaxTemps = MAX_PROGRAM_TEMPS;
   prog->MaxEnvParams = MAX_PROGRAM_ENV_PARAMS;
   prog->MaxLocalParams = MAX_PROGRAM_LOCAL_PARAMS;
   prog->MaxAddressOffset = MAX_PROGRAM_LOCAL_PARAMS;

   switch (stage) {
   case MESA_SHADER_VERTEX:
      prog->MaxParameters = MAX_VERTEX_PROGRAM_PARAMS;
      prog->MaxAttribs = MAX_VERTEX_GENERIC_ATTRIBS;
      prog->MaxAddressRegs = MAX_VERTEX_PROGRAM_ADDRESS_REGS;
      prog->MaxUniformComponents = 4 * MAX_UNIFORMS;
      prog->MaxInputComponents = 0; /* value not used */
      prog->MaxOutputComponents = 16 * 4; /* old limit not to break tnl and swrast */
      break;
   case MESA_SHADER_FRAGMENT:
      prog->MaxParameters = MAX_FRAGMENT_PROGRAM_PARAMS;
      prog->MaxAttribs = MAX_FRAGMENT_PROGRAM_INPUTS;
      prog->MaxAddressRegs = MAX_FRAGMENT_PROGRAM_ADDRESS_REGS;
      prog->MaxUniformComponents = 4 * MAX_UNIFORMS;
      prog->MaxInputComponents = 16 * 4; /* old limit not to break tnl and swrast */
      prog->MaxOutputComponents = 0; /* value not used */
      break;
   case MESA_SHADER_TESS_CTRL:
   case MESA_SHADER_TESS_EVAL:
   case MESA_SHADER_GEOMETRY:
      prog->MaxParameters = MAX_VERTEX_PROGRAM_PARAMS;
      prog->MaxAttribs = MAX_VERTEX_GENERIC_ATTRIBS;
      prog->MaxAddressRegs = MAX_VERTEX_PROGRAM_ADDRESS_REGS;
      prog->MaxUniformComponents = 4 * MAX_UNIFORMS;
      prog->MaxInputComponents = 16 * 4; /* old limit not to break tnl and swrast */
      prog->MaxOutputComponents = 16 * 4; /* old limit not to break tnl and swrast */
      break;
   case MESA_SHADER_COMPUTE:
      prog->MaxParameters = 0; /* not meaningful for compute shaders */
      prog->MaxAttribs = 0; /* not meaningful for compute shaders */
      prog->MaxAddressRegs = 0; /* not meaningful for compute shaders */
      prog->MaxUniformComponents = 4 * MAX_UNIFORMS;
      prog->MaxInputComponents = 0; /* not meaningful for compute shaders */
      prog->MaxOutputComponents = 0; /* not meaningful for compute shaders */
      break;
   default:
      assert(0 && "Bad shader stage in init_program_limits()");
   }

   /* Set the native limits to zero.  This implies that there is no native
    * support for shaders.  Let the drivers fill in the actual values.
    */
   prog->MaxNativeInstructions = 0;
   prog->MaxNativeAluInstructions = 0;
   prog->MaxNativeTexInstructions = 0;
   prog->MaxNativeTexIndirections = 0;
   prog->MaxNativeAttribs = 0;
   prog->MaxNativeTemps = 0;
   prog->MaxNativeAddressRegs = 0;
   prog->MaxNativeParameters = 0;

   /* Set GLSL datatype range/precision info assuming IEEE float values.
    * Drivers should override these defaults as needed.
    */
   prog->MediumFloat.RangeMin = 127;
   prog->MediumFloat.RangeMax = 127;
   prog->MediumFloat.Precision = 23;
   prog->LowFloat = prog->HighFloat = prog->MediumFloat;

   /* Assume ints are stored as floats for now, since this is the least-common
    * denominator.  The OpenGL ES spec implies (page 132) that the precision
    * of integer types should be 0.  Practically speaking, IEEE
    * single-precision floating point values can only store integers in the
    * range [-0x01000000, 0x01000000] without loss of precision.
    */
   prog->MediumInt.RangeMin = 24;
   prog->MediumInt.RangeMax = 24;
   prog->MediumInt.Precision = 0;
   prog->LowInt = prog->HighInt = prog->MediumInt;

   prog->MaxUniformBlocks = 12;
   prog->MaxCombinedUniformComponents = (prog->MaxUniformComponents +
                                         consts->MaxUniformBlockSize / 4 *
                                         prog->MaxUniformBlocks);

   prog->MaxAtomicBuffers = 0;
   prog->MaxAtomicCounters = 0;

   prog->MaxShaderStorageBlocks = 8;
}


/**
 * Initialize fields of gl_constants (aka ctx->Const.*).
 * Use defaults from config.h.  The device drivers will often override
 * some of these values (such as number of texture units).
 */
void
_mesa_init_constants(struct gl_constants *consts, gl_api api)
{
   int i;
   assert(consts);

   /* Constants, may be overriden (usually only reduced) by device drivers */
   consts->MaxTextureMbytes = MAX_TEXTURE_MBYTES;
   consts->MaxTextureSize = 1 << (MAX_TEXTURE_LEVELS - 1);
   consts->Max3DTextureLevels = MAX_3D_TEXTURE_LEVELS;
   consts->MaxCubeTextureLevels = MAX_CUBE_TEXTURE_LEVELS;
   consts->MaxTextureRectSize = MAX_TEXTURE_RECT_SIZE;
   consts->MaxArrayTextureLayers = MAX_ARRAY_TEXTURE_LAYERS;
   consts->MaxTextureCoordUnits = MAX_TEXTURE_COORD_UNITS;
   consts->Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits = MAX_TEXTURE_IMAGE_UNITS;
   consts->MaxTextureUnits = MIN2(consts->MaxTextureCoordUnits,
                                     consts->Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits);
   consts->MaxTextureMaxAnisotropy = MAX_TEXTURE_MAX_ANISOTROPY;
   consts->MaxTextureLodBias = MAX_TEXTURE_LOD_BIAS;
   consts->MaxTextureBufferSize = 65536;
   consts->TextureBufferOffsetAlignment = 1;
   consts->MaxArrayLockSize = MAX_ARRAY_LOCK_SIZE;
   consts->SubPixelBits = SUB_PIXEL_BITS;
   consts->MinPointSize = MIN_POINT_SIZE;
   consts->MaxPointSize = MAX_POINT_SIZE;
   consts->MinPointSizeAA = MIN_POINT_SIZE;
   consts->MaxPointSizeAA = MAX_POINT_SIZE;
   consts->PointSizeGranularity = (GLfloat) POINT_SIZE_GRANULARITY;
   consts->MinLineWidth = MIN_LINE_WIDTH;
   consts->MaxLineWidth = MAX_LINE_WIDTH;
   consts->MinLineWidthAA = MIN_LINE_WIDTH;
   consts->MaxLineWidthAA = MAX_LINE_WIDTH;
   consts->LineWidthGranularity = (GLfloat) LINE_WIDTH_GRANULARITY;
   consts->MaxClipPlanes = 6;
   consts->MaxLights = MAX_LIGHTS;
   consts->MaxShininess = 128.0;
   consts->MaxSpotExponent = 128.0;
   consts->MaxViewportWidth = 16384;
   consts->MaxViewportHeight = 16384;
   consts->MinMapBufferAlignment = 64;

   /* Driver must override these values if ARB_viewport_array is supported. */
   consts->MaxViewports = 1;
   consts->ViewportSubpixelBits = 0;
   consts->ViewportBounds.Min = 0;
   consts->ViewportBounds.Max = 0;

   /** GL_ARB_uniform_buffer_object */
   consts->MaxCombinedUniformBlocks = 36;
   consts->MaxUniformBufferBindings = 36;
   consts->MaxUniformBlockSize = 16384;
   consts->UniformBufferOffsetAlignment = 1;

   /** GL_ARB_shader_storage_buffer_object */
   consts->MaxCombinedShaderStorageBlocks = 8;
   consts->MaxShaderStorageBufferBindings = 8;
   consts->MaxShaderStorageBlockSize = 128 * 1024 * 1024; /* 2^27 */
   consts->ShaderStorageBufferOffsetAlignment = 256;

   /* GL_ARB_explicit_uniform_location, GL_MAX_UNIFORM_LOCATIONS */
   consts->MaxUserAssignableUniformLocations =
      4 * MESA_SHADER_STAGES * MAX_UNIFORMS;

   for (i = 0; i < MESA_SHADER_STAGES; i++)
      init_program_limits(consts, i, &consts->Program[i]);

   consts->MaxProgramMatrices = MAX_PROGRAM_MATRICES;
   consts->MaxProgramMatrixStackDepth = MAX_PROGRAM_MATRIX_STACK_DEPTH;

   /* Set the absolute minimum possible GLSL version.  API_OPENGL_CORE can
    * mean an OpenGL 3.0 forward-compatible context, so that implies a minimum
    * possible version of 1.30.  Otherwise, the minimum possible version 1.20.
    * Since Mesa unconditionally advertises GL_ARB_shading_language_100 and
    * GL_ARB_shader_objects, every driver has GLSL 1.20... even if they don't
    * advertise any extensions to enable any shader stages (e.g.,
    * GL_ARB_vertex_shader).
    */
   consts->GLSLVersion = api == API_OPENGL_CORE ? 130 : 120;
   consts->GLSLVersionCompat = consts->GLSLVersion;

   consts->GLSLLowerConstArrays = true;

   /* Assume that if GLSL 1.30+ (or GLSL ES 3.00+) is supported that
    * gl_VertexID is implemented using a native hardware register with OpenGL
    * semantics.
    */
   consts->VertexID_is_zero_based = false;

   /* GL_ARB_draw_buffers */
   consts->MaxDrawBuffers = MAX_DRAW_BUFFERS;

   consts->MaxColorAttachments = MAX_COLOR_ATTACHMENTS;
   consts->MaxRenderbufferSize = MAX_RENDERBUFFER_SIZE;

   consts->Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = MAX_TEXTURE_IMAGE_UNITS;
   consts->MaxCombinedTextureImageUnits = MAX_COMBINED_TEXTURE_IMAGE_UNITS;
   consts->MaxVarying = 16; /* old limit not to break tnl and swrast */
   consts->Program[MESA_SHADER_GEOMETRY].MaxTextureImageUnits = MAX_TEXTURE_IMAGE_UNITS;
   consts->MaxGeometryOutputVertices = MAX_GEOMETRY_OUTPUT_VERTICES;
   consts->MaxGeometryTotalOutputComponents = MAX_GEOMETRY_TOTAL_OUTPUT_COMPONENTS;
   consts->MaxGeometryShaderInvocations = MAX_GEOMETRY_SHADER_INVOCATIONS;

#ifdef DEBUG
   consts->GenerateTemporaryNames = true;
#else
   consts->GenerateTemporaryNames = false;
#endif

   /* GL_ARB_framebuffer_object */
   consts->MaxSamples = 0;

   /* GLSL default if NativeIntegers == FALSE */
   consts->UniformBooleanTrue = FLOAT_AS_UNION(1.0f).u;

   /* GL_ARB_sync */
   consts->MaxServerWaitTimeout = 0x7fffffff7fffffffULL;

   /* GL_EXT_provoking_vertex */
   consts->QuadsFollowProvokingVertexConvention = GL_TRUE;

   /** GL_ARB_viewport_array */
   consts->LayerAndVPIndexProvokingVertex = GL_UNDEFINED_VERTEX;

   /* GL_EXT_transform_feedback */
   consts->MaxTransformFeedbackBuffers = MAX_FEEDBACK_BUFFERS;
   consts->MaxTransformFeedbackSeparateComponents = 4 * MAX_FEEDBACK_ATTRIBS;
   consts->MaxTransformFeedbackInterleavedComponents = 4 * MAX_FEEDBACK_ATTRIBS;
   consts->MaxVertexStreams = 1;

   /* GL 3.2  */
   consts->ProfileMask = api == API_OPENGL_CORE
                          ? GL_CONTEXT_CORE_PROFILE_BIT
                          : GL_CONTEXT_COMPATIBILITY_PROFILE_BIT;

   /* GL 4.4 */
   consts->MaxVertexAttribStride = 2048;

   /** GL_EXT_gpu_shader4 */
   consts->MinProgramTexelOffset = -8;
   consts->MaxProgramTexelOffset = 7;

   /* GL_ARB_texture_gather */
   consts->MinProgramTextureGatherOffset = -8;
   consts->MaxProgramTextureGatherOffset = 7;

   /* GL_ARB_robustness */
   consts->ResetStrategy = GL_NO_RESET_NOTIFICATION_ARB;

   /* GL_KHR_robustness */
   consts->RobustAccess = GL_FALSE;

   /* ES 3.0 or ARB_ES3_compatibility */
   consts->MaxElementIndex = 0xffffffffu;

   /* GL_ARB_texture_multisample */
   consts->MaxColorTextureSamples = 1;
   consts->MaxDepthTextureSamples = 1;
   consts->MaxIntegerSamples = 1;

   /* GL_ARB_shader_atomic_counters */
   consts->MaxAtomicBufferBindings = MAX_COMBINED_ATOMIC_BUFFERS;
   consts->MaxAtomicBufferSize = MAX_ATOMIC_COUNTERS * ATOMIC_COUNTER_SIZE;
   consts->MaxCombinedAtomicBuffers = MAX_COMBINED_ATOMIC_BUFFERS;
   consts->MaxCombinedAtomicCounters = MAX_ATOMIC_COUNTERS;

   /* GL_ARB_vertex_attrib_binding */
   consts->MaxVertexAttribRelativeOffset = 2047;
   consts->MaxVertexAttribBindings = MAX_VERTEX_GENERIC_ATTRIBS;

   /* GL_ARB_compute_shader */
   consts->MaxComputeWorkGroupCount[0] = 65535;
   consts->MaxComputeWorkGroupCount[1] = 65535;
   consts->MaxComputeWorkGroupCount[2] = 65535;
   consts->MaxComputeWorkGroupSize[0] = 1024;
   consts->MaxComputeWorkGroupSize[1] = 1024;
   consts->MaxComputeWorkGroupSize[2] = 64;
   /* Enables compute support for GLES 3.1 if >= 128 */
   consts->MaxComputeWorkGroupInvocations = 0;

   /** GL_ARB_gpu_shader5 */
   consts->MinFragmentInterpolationOffset = MIN_FRAGMENT_INTERPOLATION_OFFSET;
   consts->MaxFragmentInterpolationOffset = MAX_FRAGMENT_INTERPOLATION_OFFSET;

   /** GL_KHR_context_flush_control */
   consts->ContextReleaseBehavior = GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH;

   /** GL_ARB_tessellation_shader */
   consts->MaxTessGenLevel = MAX_TESS_GEN_LEVEL;
   consts->MaxPatchVertices = MAX_PATCH_VERTICES;
   consts->Program[MESA_SHADER_TESS_CTRL].MaxTextureImageUnits = MAX_TEXTURE_IMAGE_UNITS;
   consts->Program[MESA_SHADER_TESS_EVAL].MaxTextureImageUnits = MAX_TEXTURE_IMAGE_UNITS;
   consts->MaxTessPatchComponents = MAX_TESS_PATCH_COMPONENTS;
   consts->MaxTessControlTotalOutputComponents = MAX_TESS_CONTROL_TOTAL_OUTPUT_COMPONENTS;
   consts->PrimitiveRestartForPatches = false;

   /** GL_ARB_compute_variable_group_size */
   consts->MaxComputeVariableGroupSize[0] = 512;
   consts->MaxComputeVariableGroupSize[1] = 512;
   consts->MaxComputeVariableGroupSize[2] = 64;
   consts->MaxComputeVariableGroupInvocations = 512;

   /** GL_NV_conservative_raster */
   consts->MaxSubpixelPrecisionBiasBits = 0;

   /** GL_NV_conservative_raster_dilate */
   consts->ConservativeRasterDilateRange[0] = 0.0;
   consts->ConservativeRasterDilateRange[1] = 0.0;
   consts->ConservativeRasterDilateGranularity = 0.0;

   consts->glBeginEndBufferSize = 512 * 1024;
}


/**
 * Do some sanity checks on the limits/constants for the given context.
 * Only called the first time a context is bound.
 */
static void
check_context_limits(struct gl_context *ctx)
{
   (void) ctx;

   /* check that we don't exceed the size of various bitfields */
   assert(VARYING_SLOT_MAX <=
          (8 * sizeof(ctx->VertexProgram._Current->info.outputs_written)));
   assert(VARYING_SLOT_MAX <=
          (8 * sizeof(ctx->FragmentProgram._Current->info.inputs_read)));

   /* shader-related checks */
   assert(ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxLocalParams <= MAX_PROGRAM_LOCAL_PARAMS);
   assert(ctx->Const.Program[MESA_SHADER_VERTEX].MaxLocalParams <= MAX_PROGRAM_LOCAL_PARAMS);

   /* Texture unit checks */
   assert(ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits > 0);
   assert(ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits <= MAX_TEXTURE_IMAGE_UNITS);
   assert(ctx->Const.MaxTextureCoordUnits > 0);
   assert(ctx->Const.MaxTextureCoordUnits <= MAX_TEXTURE_COORD_UNITS);
   assert(ctx->Const.MaxTextureUnits > 0);
   assert(ctx->Const.MaxTextureUnits <= MAX_TEXTURE_IMAGE_UNITS);
   assert(ctx->Const.MaxTextureUnits <= MAX_TEXTURE_COORD_UNITS);
   assert(ctx->Const.MaxTextureUnits == MIN2(ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits,
                                             ctx->Const.MaxTextureCoordUnits));
   assert(ctx->Const.MaxCombinedTextureImageUnits > 0);
   assert(ctx->Const.MaxCombinedTextureImageUnits <= MAX_COMBINED_TEXTURE_IMAGE_UNITS);
   assert(ctx->Const.MaxTextureCoordUnits <= MAX_COMBINED_TEXTURE_IMAGE_UNITS);
   /* number of coord units cannot be greater than number of image units */
   assert(ctx->Const.MaxTextureCoordUnits <= ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits);


   /* Texture size checks */
   assert(ctx->Const.MaxTextureSize <= (1 << (MAX_TEXTURE_LEVELS - 1)));
   assert(ctx->Const.Max3DTextureLevels <= MAX_3D_TEXTURE_LEVELS);
   assert(ctx->Const.MaxCubeTextureLevels <= MAX_CUBE_TEXTURE_LEVELS);
   assert(ctx->Const.MaxTextureRectSize <= MAX_TEXTURE_RECT_SIZE);

   /* Texture level checks */
   assert(MAX_TEXTURE_LEVELS >= MAX_3D_TEXTURE_LEVELS);
   assert(MAX_TEXTURE_LEVELS >= MAX_CUBE_TEXTURE_LEVELS);

   /* Max texture size should be <= max viewport size (render to texture) */
   assert(ctx->Const.MaxTextureSize <= ctx->Const.MaxViewportWidth);
   assert(ctx->Const.MaxTextureSize <= ctx->Const.MaxViewportHeight);

   assert(ctx->Const.MaxDrawBuffers <= MAX_DRAW_BUFFERS);

   /* if this fails, add more enum values to gl_buffer_index */
   assert(BUFFER_COLOR0 + MAX_DRAW_BUFFERS <= BUFFER_COUNT);

   /* XXX probably add more tests */
}


/**
 * Initialize the attribute groups in a GL context.
 *
 * \param ctx GL context.
 *
 * Initializes all the attributes, calling the respective <tt>init*</tt>
 * functions for the more complex data structures.
 */
static GLboolean
init_attrib_groups(struct gl_context *ctx)
{
   assert(ctx);

   /* Constants */
   _mesa_init_constants(&ctx->Const, ctx->API);

   /* Extensions */
   _mesa_init_extensions(&ctx->Extensions);

   /* Attribute Groups */
   _mesa_init_accum( ctx );
   _mesa_init_attrib( ctx );
   _mesa_init_bbox( ctx );
   _mesa_init_buffer_objects( ctx );
   _mesa_init_color( ctx );
   _mesa_init_conservative_raster( ctx );
   _mesa_init_current( ctx );
   _mesa_init_depth( ctx );
   _mesa_init_debug( ctx );
   _mesa_init_debug_output( ctx );
   _mesa_init_display_list( ctx );
   _mesa_init_eval( ctx );
   _mesa_init_fbobjects( ctx );
   _mesa_init_feedback( ctx );
   _mesa_init_fog( ctx );
   _mesa_init_hint( ctx );
   _mesa_init_image_units( ctx );
   _mesa_init_line( ctx );
   _mesa_init_lighting( ctx );
   _mesa_init_matrix( ctx );
   _mesa_init_multisample( ctx );
   _mesa_init_performance_monitors( ctx );
   _mesa_init_performance_queries( ctx );
   _mesa_init_pipeline( ctx );
   _mesa_init_pixel( ctx );
   _mesa_init_pixelstore( ctx );
   _mesa_init_point( ctx );
   _mesa_init_polygon( ctx );
   _mesa_init_program( ctx );
   _mesa_init_queryobj( ctx );
   _mesa_init_sync( ctx );
   _mesa_init_rastpos( ctx );
   _mesa_init_scissor( ctx );
   _mesa_init_shader_state( ctx );
   _mesa_init_stencil( ctx );
   _mesa_init_transform( ctx );
   _mesa_init_transform_feedback( ctx );
   _mesa_init_varray( ctx );
   _mesa_init_viewport( ctx );
   _mesa_init_resident_handles( ctx );

   if (!_mesa_init_texture( ctx ))
      return GL_FALSE;

   /* Miscellaneous */
   ctx->TileRasterOrderIncreasingX = GL_TRUE;
   ctx->TileRasterOrderIncreasingY = GL_TRUE;
   ctx->NewState = _NEW_ALL;
   ctx->NewDriverState = ~0;
   ctx->ErrorValue = GL_NO_ERROR;
   ctx->ShareGroupReset = false;
   ctx->varying_vp_inputs = VERT_BIT_ALL;

   return GL_TRUE;
}


/**
 * Update default objects in a GL context with respect to shared state.
 *
 * \param ctx GL context.
 *
 * Removes references to old default objects, (texture objects, program
 * objects, etc.) and changes to reference those from the current shared
 * state.
 */
static GLboolean
update_default_objects(struct gl_context *ctx)
{
   assert(ctx);

   _mesa_update_default_objects_program(ctx);
   _mesa_update_default_objects_texture(ctx);
   _mesa_update_default_objects_buffer_objects(ctx);

   return GL_TRUE;
}


/* XXX this is temporary and should be removed at some point in the
 * future when there's a reasonable expectation that the libGL library
 * contains the _glapi_new_nop_table() and _glapi_set_nop_handler()
 * functions which were added in Mesa 10.6.
 */
#if !defined(_WIN32)
/* Avoid libGL / driver ABI break */
#define USE_GLAPI_NOP_FEATURES 0
#else
#define USE_GLAPI_NOP_FEATURES 1
#endif


/**
 * This function is called by the glapi no-op functions.  For each OpenGL
 * function/entrypoint there's a simple no-op function.  These "no-op"
 * functions call this function.
 *
 * If there's a current OpenGL context for the calling thread, we record a
 * GL_INVALID_OPERATION error.  This can happen either because the app's
 * calling an unsupported extension function, or calling an illegal function
 * (such as glClear between glBegin/glEnd).
 *
 * If there's no current OpenGL context for the calling thread, we can
 * print a message to stderr.
 *
 * \param name  the name of the OpenGL function
 */
#if USE_GLAPI_NOP_FEATURES
static void
nop_handler(const char *name)
{
   GET_CURRENT_CONTEXT(ctx);
   if (ctx) {
      _mesa_error(ctx, GL_INVALID_OPERATION, "%s(invalid call)", name);
   }
#ifndef NDEBUG
   else if (getenv("MESA_DEBUG") || getenv("LIBGL_DEBUG")) {
      fprintf(stderr,
              "GL User Error: gl%s called without a rendering context\n",
              name);
      fflush(stderr);
   }
#endif
}
#endif


/**
 * Special no-op glFlush, see below.
 */
#if defined(_WIN32)
static void GLAPIENTRY
nop_glFlush(void)
{
   /* don't record an error like we do in nop_handler() */
}
#endif


#if !USE_GLAPI_NOP_FEATURES
static int
generic_nop(void)
{
   GET_CURRENT_CONTEXT(ctx);
   _mesa_error(ctx, GL_INVALID_OPERATION,
               "unsupported function called "
               "(unsupported extension or deprecated function?)");
   return 0;
}
#endif


/**
 * Create a new API dispatch table in which all entries point to the
 * generic_nop() function.  This will not work on Windows because of
 * the __stdcall convention which requires the callee to clean up the
 * call stack.  That's impossible with one generic no-op function.
 */
struct _glapi_table *
_mesa_new_nop_table(unsigned numEntries)
{
   struct _glapi_table *table;

#if !USE_GLAPI_NOP_FEATURES
   table = malloc(numEntries * sizeof(_glapi_proc));
   if (table) {
      _glapi_proc *entry = (_glapi_proc *) table;
      unsigned i;
      for (i = 0; i < numEntries; i++) {
         entry[i] = (_glapi_proc) generic_nop;
      }
   }
#else
   table = _glapi_new_nop_table(numEntries);
#endif
   return table;
}


/**
 * Allocate and initialize a new dispatch table.  The table will be
 * populated with pointers to "no-op" functions.  In turn, the no-op
 * functions will call nop_handler() above.
 */
struct _glapi_table *
_mesa_alloc_dispatch_table(void)
{
   /* Find the larger of Mesa's dispatch table and libGL's dispatch table.
    * In practice, this'll be the same for stand-alone Mesa.  But for DRI
    * Mesa we do this to accommodate different versions of libGL and various
    * DRI drivers.
    */
   int numEntries = MAX2(_glapi_get_dispatch_table_size(), _gloffset_COUNT);

   struct _glapi_table *table = _mesa_new_nop_table(numEntries);

#if defined(_WIN32)
   if (table) {
      /* This is a special case for Windows in the event that
       * wglGetProcAddress is called between glBegin/End().
       *
       * The MS opengl32.dll library apparently calls glFlush from
       * wglGetProcAddress().  If we're inside glBegin/End(), glFlush
       * will dispatch to _mesa_generic_nop() and we'll generate a
       * GL_INVALID_OPERATION error.
       *
       * The specific case which hits this is piglit's primitive-restart
       * test which calls glPrimitiveRestartNV() inside glBegin/End.  The
       * first time we call glPrimitiveRestartNV() Piglit's API dispatch
       * code will try to resolve the function by calling wglGetProcAddress.
       * This raises GL_INVALID_OPERATION and an assert(glGetError()==0)
       * will fail causing the test to fail.  By suppressing the error, the
       * assertion passes and the test continues.
       */
      SET_Flush(table, nop_glFlush);
   }
#endif

#if USE_GLAPI_NOP_FEATURES
   _glapi_set_nop_handler(nop_handler);
#endif

   return table;
}

/**
 * Creates a minimal dispatch table for use within glBegin()/glEnd().
 *
 * This ensures that we generate GL_INVALID_OPERATION errors from most
 * functions, since the set of functions that are valid within Begin/End is
 * very small.
 *
 * From the GL 1.0 specification section 2.6.3, "GL Commands within
 * Begin/End"
 *
 *     "The only GL commands that are allowed within any Begin/End pairs are
 *      the commands for specifying vertex coordinates, vertex color, normal
 *      coordinates, and texture coordinates (Vertex, Color, Index, Normal,
 *      TexCoord), EvalCoord and EvalPoint commands (see section 5.1),
 *      commands for specifying lighting material parameters (Material
 *      commands see section 2.12.2), display list invocation commands
 *      (CallList and CallLists see section 5.4), and the EdgeFlag
 *      command. Executing Begin after Begin has already been executed but
 *      before an End is issued generates the INVALID OPERATION error, as does
 *      executing End without a previous corresponding Begin. Executing any
 *      other GL command within Begin/End results in the error INVALID
 *      OPERATION."
 *
 * The table entries for specifying vertex attributes are set up by
 * install_vtxfmt() and _mesa_loopback_init_api_table(), and End() and dlists
 * are set by install_vtxfmt() as well.
 */
static struct _glapi_table *
create_beginend_table(const struct gl_context *ctx)
{
   struct _glapi_table *table;

   table = _mesa_alloc_dispatch_table();
   if (!table)
      return NULL;

   /* Fill in functions which return a value, since they should return some
    * specific value even if they emit a GL_INVALID_OPERATION error from them
    * being called within glBegin()/glEnd().
    */
#define COPY_DISPATCH(func) SET_##func(table, GET_##func(ctx->Exec))

   COPY_DISPATCH(GenLists);
   COPY_DISPATCH(IsProgram);
   COPY_DISPATCH(IsVertexArray);
   COPY_DISPATCH(IsBuffer);
   COPY_DISPATCH(IsEnabled);
   COPY_DISPATCH(IsEnabledi);
   COPY_DISPATCH(IsRenderbuffer);
   COPY_DISPATCH(IsFramebuffer);
   COPY_DISPATCH(CheckFramebufferStatus);
   COPY_DISPATCH(RenderMode);
   COPY_DISPATCH(GetString);
   COPY_DISPATCH(GetStringi);
   COPY_DISPATCH(GetPointerv);
   COPY_DISPATCH(IsQuery);
   COPY_DISPATCH(IsSampler);
   COPY_DISPATCH(IsSync);
   COPY_DISPATCH(IsTexture);
   COPY_DISPATCH(IsTransformFeedback);
   COPY_DISPATCH(DeleteQueries);
   COPY_DISPATCH(AreTexturesResident);
   COPY_DISPATCH(FenceSync);
   COPY_DISPATCH(ClientWaitSync);
   COPY_DISPATCH(MapBuffer);
   COPY_DISPATCH(UnmapBuffer);
   COPY_DISPATCH(MapBufferRange);
   COPY_DISPATCH(ObjectPurgeableAPPLE);
   COPY_DISPATCH(ObjectUnpurgeableAPPLE);

   _mesa_loopback_init_api_table(ctx, table);

   return table;
}

void
_mesa_initialize_dispatch_tables(struct gl_context *ctx)
{
   /* Do the code-generated setup of the exec table in api_exec.c. */
   _mesa_initialize_exec_table(ctx);

   if (ctx->Save)
      _mesa_initialize_save_table(ctx);
}

/**
 * Initialize a struct gl_context struct (rendering context).
 *
 * This includes allocating all the other structs and arrays which hang off of
 * the context by pointers.
 * Note that the driver needs to pass in its dd_function_table here since
 * we need to at least call driverFunctions->NewTextureObject to create the
 * default texture objects.
 *
 * Called by _mesa_create_context().
 *
 * Performs the imports and exports callback tables initialization, and
 * miscellaneous one-time initializations. If no shared context is supplied one
 * is allocated, and increase its reference count.  Setups the GL API dispatch
 * tables.  Initialize the TNL module. Sets the maximum Z buffer depth.
 * Finally queries the \c MESA_DEBUG and \c MESA_VERBOSE environment variables
 * for debug flags.
 *
 * \param ctx the context to initialize
 * \param api the GL API type to create the context for
 * \param visual describes the visual attributes for this context or NULL to
 *               create a configless context
 * \param share_list points to context to share textures, display lists,
 *        etc with, or NULL
 * \param driverFunctions table of device driver functions for this context
 *        to use
 */
GLboolean
_mesa_initialize_context(struct gl_context *ctx,
                         gl_api api,
                         const struct gl_config *visual,
                         struct gl_context *share_list,
                         const struct dd_function_table *driverFunctions)
{
   struct gl_shared_state *shared;
   int i;

   assert(driverFunctions->NewTextureObject);
   assert(driverFunctions->FreeTextureImageBuffer);

   ctx->API = api;
   ctx->DrawBuffer = NULL;
   ctx->ReadBuffer = NULL;
   ctx->WinSysDrawBuffer = NULL;
   ctx->WinSysReadBuffer = NULL;

   if (visual) {
      ctx->Visual = *visual;
      ctx->HasConfig = GL_TRUE;
   }
   else {
      memset(&ctx->Visual, 0, sizeof ctx->Visual);
      ctx->HasConfig = GL_FALSE;
   }

   _mesa_override_gl_version(ctx);

   /* misc one-time initializations */
   _mesa_initialize();

   /* Plug in driver functions and context pointer here.
    * This is important because when we call alloc_shared_state() below
    * we'll call ctx->Driver.NewTextureObject() to create the default
    * textures.
    */
   ctx->Driver = *driverFunctions;

   if (share_list) {
      /* share state with another context */
      shared = share_list->Shared;
   }
   else {
      /* allocate new, unshared state */
      shared = _mesa_alloc_shared_state(ctx);
      if (!shared)
         return GL_FALSE;
   }

   _mesa_reference_shared_state(ctx, &ctx->Shared, shared);

   if (!init_attrib_groups( ctx ))
      goto fail;

   /* KHR_no_error is likely to crash, overflow memory, etc if an application
    * has errors so don't enable it for setuid processes.
    */
   if (env_var_as_boolean("MESA_NO_ERROR", false)) {
#if !defined(_WIN32)
      if (geteuid() == getuid())
#endif
         ctx->Const.ContextFlags |= GL_CONTEXT_FLAG_NO_ERROR_BIT_KHR;
   }

   /* setup the API dispatch tables with all nop functions */
   ctx->OutsideBeginEnd = _mesa_alloc_dispatch_table();
   if (!ctx->OutsideBeginEnd)
      goto fail;
   ctx->Exec = ctx->OutsideBeginEnd;
   ctx->CurrentClientDispatch = ctx->CurrentServerDispatch = ctx->OutsideBeginEnd;

   ctx->FragmentProgram._MaintainTexEnvProgram
      = (getenv("MESA_TEX_PROG") != NULL);

   ctx->VertexProgram._MaintainTnlProgram
      = (getenv("MESA_TNL_PROG") != NULL);
   if (ctx->VertexProgram._MaintainTnlProgram) {
      /* this is required... */
      ctx->FragmentProgram._MaintainTexEnvProgram = GL_TRUE;
   }

   /* Mesa core handles all the formats that mesa core knows about.
    * Drivers will want to override this list with just the formats
    * they can handle, and confirm that appropriate fallbacks exist in
    * _mesa_choose_tex_format().
    */
   memset(&ctx->TextureFormatSupported, GL_TRUE,
          sizeof(ctx->TextureFormatSupported));

   switch (ctx->API) {
   case API_OPENGL_COMPAT:
      ctx->BeginEnd = create_beginend_table(ctx);
      ctx->Save = _mesa_alloc_dispatch_table();
      if (!ctx->BeginEnd || !ctx->Save)
         goto fail;

      /* fall-through */
   case API_OPENGL_CORE:
      break;
   case API_OPENGLES:
      /**
       * GL_OES_texture_cube_map says
       * "Initially all texture generation modes are set to REFLECTION_MAP_OES"
       */
      for (i = 0; i < ARRAY_SIZE(ctx->Texture.FixedFuncUnit); i++) {
         struct gl_fixedfunc_texture_unit *texUnit =
            &ctx->Texture.FixedFuncUnit[i];

         texUnit->GenS.Mode = GL_REFLECTION_MAP_NV;
         texUnit->GenT.Mode = GL_REFLECTION_MAP_NV;
         texUnit->GenR.Mode = GL_REFLECTION_MAP_NV;
         texUnit->GenS._ModeBit = TEXGEN_REFLECTION_MAP_NV;
         texUnit->GenT._ModeBit = TEXGEN_REFLECTION_MAP_NV;
         texUnit->GenR._ModeBit = TEXGEN_REFLECTION_MAP_NV;
      }
      break;
   case API_OPENGLES2:
      ctx->FragmentProgram._MaintainTexEnvProgram = GL_TRUE;
      ctx->VertexProgram._MaintainTnlProgram = GL_TRUE;
      break;
   }

   ctx->FirstTimeCurrent = GL_TRUE;

   return GL_TRUE;

fail:
   _mesa_reference_shared_state(ctx, &ctx->Shared, NULL);
   free(ctx->BeginEnd);
   free(ctx->OutsideBeginEnd);
   free(ctx->Save);
   return GL_FALSE;
}


/**
 * Free the data associated with the given context.
 *
 * But doesn't free the struct gl_context struct itself.
 *
 * \sa _mesa_initialize_context() and init_attrib_groups().
 */
void
_mesa_free_context_data(struct gl_context *ctx)
{
   if (!_mesa_get_current_context()){
      /* No current context, but we may need one in order to delete
       * texture objs, etc.  So temporarily bind the context now.
       */
      _mesa_make_current(ctx, NULL, NULL);
   }

   /* unreference WinSysDraw/Read buffers */
   _mesa_reference_framebuffer(&ctx->WinSysDrawBuffer, NULL);
   _mesa_reference_framebuffer(&ctx->WinSysReadBuffer, NULL);
   _mesa_reference_framebuffer(&ctx->DrawBuffer, NULL);
   _mesa_reference_framebuffer(&ctx->ReadBuffer, NULL);

   _mesa_reference_program(ctx, &ctx->VertexProgram.Current, NULL);
   _mesa_reference_program(ctx, &ctx->VertexProgram._Current, NULL);
   _mesa_reference_program(ctx, &ctx->VertexProgram._TnlProgram, NULL);

   _mesa_reference_program(ctx, &ctx->TessCtrlProgram._Current, NULL);
   _mesa_reference_program(ctx, &ctx->TessEvalProgram._Current, NULL);
   _mesa_reference_program(ctx, &ctx->GeometryProgram._Current, NULL);

   _mesa_reference_program(ctx, &ctx->FragmentProgram.Current, NULL);
   _mesa_reference_program(ctx, &ctx->FragmentProgram._Current, NULL);
   _mesa_reference_program(ctx, &ctx->FragmentProgram._TexEnvProgram, NULL);

   _mesa_reference_program(ctx, &ctx->ComputeProgram._Current, NULL);

   _mesa_reference_vao(ctx, &ctx->Array.VAO, NULL);
   _mesa_reference_vao(ctx, &ctx->Array.DefaultVAO, NULL);
   _mesa_reference_vao(ctx, &ctx->Array._EmptyVAO, NULL);
   _mesa_reference_vao(ctx, &ctx->Array._DrawVAO, NULL);

   _mesa_free_attrib_data(ctx);
   _mesa_free_buffer_objects(ctx);
   _mesa_free_eval_data( ctx );
   _mesa_free_texture_data( ctx );
   _mesa_free_image_textures(ctx);
   _mesa_free_matrix_data( ctx );
   _mesa_free_pipeline_data(ctx);
   _mesa_free_program_data(ctx);
   _mesa_free_shader_state(ctx);
   _mesa_free_queryobj_data(ctx);
   _mesa_free_sync_data(ctx);
   _mesa_free_varray_data(ctx);
   _mesa_free_transform_feedback(ctx);
   _mesa_free_performance_monitors(ctx);
   _mesa_free_performance_queries(ctx);
   _mesa_free_resident_handles(ctx);

   _mesa_reference_buffer_object(ctx, &ctx->Pack.BufferObj, NULL);
   _mesa_reference_buffer_object(ctx, &ctx->Unpack.BufferObj, NULL);
   _mesa_reference_buffer_object(ctx, &ctx->DefaultPacking.BufferObj, NULL);
   _mesa_reference_buffer_object(ctx, &ctx->Array.ArrayBufferObj, NULL);

   /* free dispatch tables */
   free(ctx->BeginEnd);
   free(ctx->OutsideBeginEnd);
   free(ctx->Save);
   free(ctx->ContextLost);
   free(ctx->MarshalExec);

   /* Shared context state (display lists, textures, etc) */
   _mesa_reference_shared_state(ctx, &ctx->Shared, NULL);

   /* needs to be after freeing shared state */
   _mesa_free_display_list_data(ctx);

   _mesa_free_errors_data(ctx);

   free((void *)ctx->Extensions.String);

   free(ctx->VersionString);

   ralloc_free(ctx->SoftFP64);

   /* unbind the context if it's currently bound */
   if (ctx == _mesa_get_current_context()) {
      _mesa_make_current(NULL, NULL, NULL);
   }

   /* Do this after unbinding context to ensure any thread is finished. */
   if (ctx->shader_builtin_ref) {
      _mesa_glsl_builtin_functions_decref();
      ctx->shader_builtin_ref = false;
   }

   free(ctx->Const.SpirVExtensions);
}


/**
 * Destroy a struct gl_context structure.
 *
 * \param ctx GL context.
 *
 * Calls _mesa_free_context_data() and frees the gl_context object itself.
 */
void
_mesa_destroy_context( struct gl_context *ctx )
{
   if (ctx) {
      _mesa_free_context_data(ctx);
      free( (void *) ctx );
   }
}


/**
 * Copy attribute groups from one context to another.
 *
 * \param src source context
 * \param dst destination context
 * \param mask bitwise OR of GL_*_BIT flags
 *
 * According to the bits specified in \p mask, copies the corresponding
 * attributes from \p src into \p dst.  For many of the attributes a simple \c
 * memcpy is not enough due to the existence of internal pointers in their data
 * structures.
 */
void
_mesa_copy_context( const struct gl_context *src, struct gl_context *dst,
                    GLuint mask )
{
   if (mask & GL_ACCUM_BUFFER_BIT) {
      /* OK to memcpy */
      dst->Accum = src->Accum;
   }
   if (mask & GL_COLOR_BUFFER_BIT) {
      /* OK to memcpy */
      dst->Color = src->Color;
   }
   if (mask & GL_CURRENT_BIT) {
      /* OK to memcpy */
      dst->Current = src->Current;
   }
   if (mask & GL_DEPTH_BUFFER_BIT) {
      /* OK to memcpy */
      dst->Depth = src->Depth;
   }
   if (mask & GL_ENABLE_BIT) {
      /* no op */
   }
   if (mask & GL_EVAL_BIT) {
      /* OK to memcpy */
      dst->Eval = src->Eval;
   }
   if (mask & GL_FOG_BIT) {
      /* OK to memcpy */
      dst->Fog = src->Fog;
   }
   if (mask & GL_HINT_BIT) {
      /* OK to memcpy */
      dst->Hint = src->Hint;
   }
   if (mask & GL_LIGHTING_BIT) {
      /* OK to memcpy */
      dst->Light = src->Light;
   }
   if (mask & GL_LINE_BIT) {
      /* OK to memcpy */
      dst->Line = src->Line;
   }
   if (mask & GL_LIST_BIT) {
      /* OK to memcpy */
      dst->List = src->List;
   }
   if (mask & GL_PIXEL_MODE_BIT) {
      /* OK to memcpy */
      dst->Pixel = src->Pixel;
   }
   if (mask & GL_POINT_BIT) {
      /* OK to memcpy */
      dst->Point = src->Point;
   }
   if (mask & GL_POLYGON_BIT) {
      /* OK to memcpy */
      dst->Polygon = src->Polygon;
   }
   if (mask & GL_POLYGON_STIPPLE_BIT) {
      /* Use loop instead of memcpy due to problem with Portland Group's
       * C compiler.  Reported by John Stone.
       */
      GLuint i;
      for (i = 0; i < 32; i++) {
         dst->PolygonStipple[i] = src->PolygonStipple[i];
      }
   }
   if (mask & GL_SCISSOR_BIT) {
      /* OK to memcpy */
      dst->Scissor = src->Scissor;
   }
   if (mask & GL_STENCIL_BUFFER_BIT) {
      /* OK to memcpy */
      dst->Stencil = src->Stencil;
   }
   if (mask & GL_TEXTURE_BIT) {
      /* Cannot memcpy because of pointers */
      _mesa_copy_texture_state(src, dst);
   }
   if (mask & GL_TRANSFORM_BIT) {
      /* OK to memcpy */
      dst->Transform = src->Transform;
   }
   if (mask & GL_VIEWPORT_BIT) {
      unsigned i;
      for (i = 0; i < src->Const.MaxViewports; i++) {
         /* OK to memcpy */
         dst->ViewportArray[i] = src->ViewportArray[i];
      }
   }

   /* XXX FIXME:  Call callbacks?
    */
   dst->NewState = _NEW_ALL;
   dst->NewDriverState = ~0;
}


/**
 * Check if the given context can render into the given framebuffer
 * by checking visual attributes.
 *
 * \return GL_TRUE if compatible, GL_FALSE otherwise.
 */
static GLboolean
check_compatible(const struct gl_context *ctx,
                 const struct gl_framebuffer *buffer)
{
   const struct gl_config *ctxvis = &ctx->Visual;
   const struct gl_config *bufvis = &buffer->Visual;

   if (buffer == _mesa_get_incomplete_framebuffer())
      return GL_TRUE;

#define check_component(foo)           \
   if (ctxvis->foo && bufvis->foo &&   \
       ctxvis->foo != bufvis->foo)     \
      return GL_FALSE

   check_component(redShift);
   check_component(greenShift);
   check_component(blueShift);
   check_component(redBits);
   check_component(greenBits);
   check_component(blueBits);
   check_component(depthBits);
   check_component(stencilBits);

#undef check_component

   return GL_TRUE;
}


/**
 * Check if the viewport/scissor size has not yet been initialized.
 * Initialize the size if the given width and height are non-zero.
 */
static void
check_init_viewport(struct gl_context *ctx, GLuint width, GLuint height)
{
   if (!ctx->ViewportInitialized && width > 0 && height > 0) {
      unsigned i;

      /* Note: set flag here, before calling _mesa_set_viewport(), to prevent
       * potential infinite recursion.
       */
      ctx->ViewportInitialized = GL_TRUE;

      /* Note: ctx->Const.MaxViewports may not have been set by the driver
       * yet, so just initialize all of them.
       */
      for (i = 0; i < MAX_VIEWPORTS; i++) {
         _mesa_set_viewport(ctx, i, 0, 0, width, height);
         _mesa_set_scissor(ctx, i, 0, 0, width, height);
      }
   }
}


static void
handle_first_current(struct gl_context *ctx)
{
   if (ctx->Version == 0 || !ctx->DrawBuffer) {
      /* probably in the process of tearing down the context */
      return;
   }

   check_context_limits(ctx);

   _mesa_update_vertex_processing_mode(ctx);

   /* According to GL_MESA_configless_context the default value of
    * glDrawBuffers depends on the config of the first surface it is bound to.
    * For GLES it is always GL_BACK which has a magic interpretation.
    */
   if (!ctx->HasConfig && _mesa_is_desktop_gl(ctx)) {
      if (ctx->DrawBuffer != _mesa_get_incomplete_framebuffer()) {
         GLenum16 buffer;

         if (ctx->DrawBuffer->Visual.doubleBufferMode)
            buffer = GL_BACK;
         else
            buffer = GL_FRONT;

         _mesa_drawbuffers(ctx, ctx->DrawBuffer, 1, &buffer,
                           NULL /* destMask */);
      }

      if (ctx->ReadBuffer != _mesa_get_incomplete_framebuffer()) {
         gl_buffer_index bufferIndex;
         GLenum buffer;

         if (ctx->ReadBuffer->Visual.doubleBufferMode) {
            buffer = GL_BACK;
            bufferIndex = BUFFER_BACK_LEFT;
         }
         else {
            buffer = GL_FRONT;
            bufferIndex = BUFFER_FRONT_LEFT;
         }

         _mesa_readbuffer(ctx, ctx->ReadBuffer, buffer, bufferIndex);
      }
   }

   /* Determine if generic vertex attribute 0 aliases the conventional
    * glVertex position.
    */
   {
      const bool is_forward_compatible_context =
         ctx->Const.ContextFlags & GL_CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT;

      /* In OpenGL 3.1 attribute 0 becomes non-magic, just like in OpenGL ES
       * 2.0.  Note that we cannot just check for API_OPENGL_COMPAT here because
       * that will erroneously allow this usage in a 3.0 forward-compatible
       * context too.
       */
      ctx->_AttribZeroAliasesVertex = (ctx->API == API_OPENGLES
                                       || (ctx->API == API_OPENGL_COMPAT
                                           && !is_forward_compatible_context));
   }

   /* We can use this to help debug user's problems.  Tell them to set
    * the MESA_INFO env variable before running their app.  Then the
    * first time each context is made current we'll print some useful
    * information.
    */
   if (getenv("MESA_INFO")) {
      _mesa_print_info(ctx);
   }
}

/**
 * Bind the given context to the given drawBuffer and readBuffer and
 * make it the current context for the calling thread.
 * We'll render into the drawBuffer and read pixels from the
 * readBuffer (i.e. glRead/CopyPixels, glCopyTexImage, etc).
 *
 * We check that the context's and framebuffer's visuals are compatible
 * and return immediately if they're not.
 *
 * \param newCtx  the new GL context. If NULL then there will be no current GL
 *                context.
 * \param drawBuffer  the drawing framebuffer
 * \param readBuffer  the reading framebuffer
 */
GLboolean
_mesa_make_current( struct gl_context *newCtx,
                    struct gl_framebuffer *drawBuffer,
                    struct gl_framebuffer *readBuffer )
{
   GET_CURRENT_CONTEXT(curCtx);

   if (MESA_VERBOSE & VERBOSE_API)
      _mesa_debug(newCtx, "_mesa_make_current()\n");

   /* Check that the context's and framebuffer's visuals are compatible.
    */
   if (newCtx && drawBuffer && newCtx->WinSysDrawBuffer != drawBuffer) {
      if (!check_compatible(newCtx, drawBuffer)) {
         _mesa_warning(newCtx,
              "MakeCurrent: incompatible visuals for context and drawbuffer");
         return GL_FALSE;
      }
   }
   if (newCtx && readBuffer && newCtx->WinSysReadBuffer != readBuffer) {
      if (!check_compatible(newCtx, readBuffer)) {
         _mesa_warning(newCtx,
              "MakeCurrent: incompatible visuals for context and readbuffer");
         return GL_FALSE;
      }
   }

   if (curCtx &&
       (curCtx->WinSysDrawBuffer || curCtx->WinSysReadBuffer) &&
       /* make sure this context is valid for flushing */
       curCtx != newCtx &&
       curCtx->Const.ContextReleaseBehavior ==
       GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH) {
      _mesa_flush(curCtx);
   }

   /* Call this periodically to detect when the user has begun using
    * GL rendering from multiple threads.
    */
   _glapi_check_multithread();

   if (!newCtx) {
      _glapi_set_dispatch(NULL);  /* none current */
      /* We need old ctx to correctly release Draw/ReadBuffer
       * and avoid a surface leak in st_renderbuffer_delete.
       * Therefore, first drop buffers then set new ctx to NULL.
       */
      if (curCtx) {
         _mesa_reference_framebuffer(&curCtx->WinSysDrawBuffer, NULL);
         _mesa_reference_framebuffer(&curCtx->WinSysReadBuffer, NULL);
      }
      _glapi_set_context(NULL);
      assert(_mesa_get_current_context() == NULL);
   }
   else {
      _glapi_set_context((void *) newCtx);
      assert(_mesa_get_current_context() == newCtx);
      _glapi_set_dispatch(newCtx->CurrentClientDispatch);

      if (drawBuffer && readBuffer) {
         assert(_mesa_is_winsys_fbo(drawBuffer));
         assert(_mesa_is_winsys_fbo(readBuffer));
         _mesa_reference_framebuffer(&newCtx->WinSysDrawBuffer, drawBuffer);
         _mesa_reference_framebuffer(&newCtx->WinSysReadBuffer, readBuffer);

         /*
          * Only set the context's Draw/ReadBuffer fields if they're NULL
          * or not bound to a user-created FBO.
          */
         if (!newCtx->DrawBuffer || _mesa_is_winsys_fbo(newCtx->DrawBuffer)) {
            _mesa_reference_framebuffer(&newCtx->DrawBuffer, drawBuffer);
            /* Update the FBO's list of drawbuffers/renderbuffers.
             * For winsys FBOs this comes from the GL state (which may have
             * changed since the last time this FBO was bound).
             */
            _mesa_update_draw_buffers(newCtx);
            _mesa_update_allow_draw_out_of_order(newCtx);
         }
         if (!newCtx->ReadBuffer || _mesa_is_winsys_fbo(newCtx->ReadBuffer)) {
            _mesa_reference_framebuffer(&newCtx->ReadBuffer, readBuffer);
            /* In _mesa_initialize_window_framebuffer, for single-buffered
             * visuals, the ColorReadBuffer is set to be GL_FRONT, even with
             * GLES contexts. When calling read_buffer, we verify we are reading
             * from GL_BACK in is_legal_es3_readbuffer_enum.  But the default is
             * incorrect, and certain dEQP tests check this.  So fix it here.
             */
            if (_mesa_is_gles(newCtx) &&
               !newCtx->ReadBuffer->Visual.doubleBufferMode)
               if (newCtx->ReadBuffer->ColorReadBuffer == GL_FRONT)
                  newCtx->ReadBuffer->ColorReadBuffer = GL_BACK;
         }

         /* XXX only set this flag if we're really changing the draw/read
          * framebuffer bindings.
          */
         newCtx->NewState |= _NEW_BUFFERS;

         check_init_viewport(newCtx, drawBuffer->Width, drawBuffer->Height);
      }

      if (newCtx->FirstTimeCurrent) {
         handle_first_current(newCtx);
         newCtx->FirstTimeCurrent = GL_FALSE;
      }
   }

   return GL_TRUE;
}


/**
 * Make context 'ctx' share the display lists, textures and programs
 * that are associated with 'ctxToShare'.
 * Any display lists, textures or programs associated with 'ctx' will
 * be deleted if nobody else is sharing them.
 */
GLboolean
_mesa_share_state(struct gl_context *ctx, struct gl_context *ctxToShare)
{
   if (ctx && ctxToShare && ctx->Shared && ctxToShare->Shared) {
      struct gl_shared_state *oldShared = NULL;

      /* save ref to old state to prevent it from being deleted immediately */
      _mesa_reference_shared_state(ctx, &oldShared, ctx->Shared);

      /* update ctx's Shared pointer */
      _mesa_reference_shared_state(ctx, &ctx->Shared, ctxToShare->Shared);

      update_default_objects(ctx);

      /* release the old shared state */
      _mesa_reference_shared_state(ctx, &oldShared, NULL);

      return GL_TRUE;
   }
   else {
      return GL_FALSE;
   }
}



/**
 * \return pointer to the current GL context for this thread.
 *
 * Calls _glapi_get_context(). This isn't the fastest way to get the current
 * context.  If you need speed, see the #GET_CURRENT_CONTEXT macro in
 * context.h.
 */
struct gl_context *
_mesa_get_current_context( void )
{
   return (struct gl_context *) _glapi_get_context();
}


/**
 * Get context's current API dispatch table.
 *
 * It'll either be the immediate-mode execute dispatcher, the display list
 * compile dispatcher, or the thread marshalling dispatcher.
 *
 * \param ctx GL context.
 *
 * \return pointer to dispatch_table.
 *
 * Simply returns __struct gl_contextRec::CurrentClientDispatch.
 */
struct _glapi_table *
_mesa_get_dispatch(struct gl_context *ctx)
{
   return ctx->CurrentClientDispatch;
}

/*@}*/


/**********************************************************************/
/** \name Miscellaneous functions                                     */
/**********************************************************************/
/*@{*/
/**
 * Flush commands.
 */
void
_mesa_flush(struct gl_context *ctx)
{
   FLUSH_VERTICES( ctx, 0 );
   if (ctx->Driver.Flush) {
      ctx->Driver.Flush(ctx);
   }
}



/**
 * Flush commands and wait for completion.
 *
 * Calls the #ASSERT_OUTSIDE_BEGIN_END_AND_FLUSH macro and the
 * dd_function_table::Finish driver callback, if not NULL.
 */
void GLAPIENTRY
_mesa_Finish(void)
{
   GET_CURRENT_CONTEXT(ctx);
   ASSERT_OUTSIDE_BEGIN_END(ctx);

   FLUSH_VERTICES(ctx, 0);

   if (ctx->Driver.Finish) {
      ctx->Driver.Finish(ctx);
   }
}


/**
 * Execute glFlush().
 *
 * Calls the #ASSERT_OUTSIDE_BEGIN_END_AND_FLUSH macro and the
 * dd_function_table::Flush driver callback, if not NULL.
 */
void GLAPIENTRY
_mesa_Flush(void)
{
   GET_CURRENT_CONTEXT(ctx);
   ASSERT_OUTSIDE_BEGIN_END(ctx);
   _mesa_flush(ctx);
}


/*@}*/
