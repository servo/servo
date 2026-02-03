/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 1999-2008  Brian Paul   All Rights Reserved.
 * Copyright (C) 2009  VMware, Inc.  All Rights Reserved.
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
 * \file mtypes.h
 * Main Mesa data structures.
 *
 * Please try to mark derived values with a leading underscore ('_').
 */

#ifndef MTYPES_H
#define MTYPES_H


#include <stdint.h>             /* uint32_t */
#include <stdbool.h>
#include "c11/threads.h"

#include "main/glheader.h"
#include "main/glthread.h"
#include "main/menums.h"
#include "main/config.h"
#include "glapi/glapi.h"
#include "math/m_matrix.h"	/* GLmatrix */
#include "compiler/shader_enums.h"
#include "compiler/shader_info.h"
#include "main/formats.h"       /* MESA_FORMAT_COUNT */
#include "compiler/glsl/list.h"
#include "util/simple_mtx.h"
#include "util/u_dynarray.h"


#ifdef __cplusplus
extern "C" {
#endif

#define GET_COLORMASK_BIT(mask, buf, chan) (((mask) >> (4 * (buf) + (chan))) & 0x1)
#define GET_COLORMASK(mask, buf) (((mask) >> (4 * (buf))) & 0xf)


/**
 * \name Some forward type declarations
 */
/*@{*/
struct _mesa_HashTable;
struct gl_attrib_node;
struct gl_list_extensions;
struct gl_meta_state;
struct gl_program_cache;
struct gl_texture_object;
struct gl_debug_state;
struct gl_context;
struct st_context;
struct gl_uniform_storage;
struct prog_instruction;
struct gl_program_parameter_list;
struct gl_shader_spirv_data;
struct set;
struct shader_includes;
struct vbo_context;
/*@}*/


/** Extra draw modes beyond GL_POINTS, GL_TRIANGLE_FAN, etc */
#define PRIM_MAX                 GL_PATCHES
#define PRIM_OUTSIDE_BEGIN_END   (PRIM_MAX + 1)
#define PRIM_UNKNOWN             (PRIM_MAX + 2)

/**
 * Determine if the given gl_varying_slot appears in the fragment shader.
 */
static inline GLboolean
_mesa_varying_slot_in_fs(gl_varying_slot slot)
{
   switch (slot) {
   case VARYING_SLOT_PSIZ:
   case VARYING_SLOT_BFC0:
   case VARYING_SLOT_BFC1:
   case VARYING_SLOT_EDGE:
   case VARYING_SLOT_CLIP_VERTEX:
   case VARYING_SLOT_LAYER:
   case VARYING_SLOT_TESS_LEVEL_OUTER:
   case VARYING_SLOT_TESS_LEVEL_INNER:
   case VARYING_SLOT_BOUNDING_BOX0:
   case VARYING_SLOT_BOUNDING_BOX1:
   case VARYING_SLOT_VIEWPORT_MASK:
      return GL_FALSE;
   default:
      return GL_TRUE;
   }
}

/**
 * Bit flags for all renderbuffers
 */
#define BUFFER_BIT_FRONT_LEFT   (1 << BUFFER_FRONT_LEFT)
#define BUFFER_BIT_BACK_LEFT    (1 << BUFFER_BACK_LEFT)
#define BUFFER_BIT_FRONT_RIGHT  (1 << BUFFER_FRONT_RIGHT)
#define BUFFER_BIT_BACK_RIGHT   (1 << BUFFER_BACK_RIGHT)
#define BUFFER_BIT_AUX0         (1 << BUFFER_AUX0)
#define BUFFER_BIT_AUX1         (1 << BUFFER_AUX1)
#define BUFFER_BIT_AUX2         (1 << BUFFER_AUX2)
#define BUFFER_BIT_AUX3         (1 << BUFFER_AUX3)
#define BUFFER_BIT_DEPTH        (1 << BUFFER_DEPTH)
#define BUFFER_BIT_STENCIL      (1 << BUFFER_STENCIL)
#define BUFFER_BIT_ACCUM        (1 << BUFFER_ACCUM)
#define BUFFER_BIT_COLOR0       (1 << BUFFER_COLOR0)
#define BUFFER_BIT_COLOR1       (1 << BUFFER_COLOR1)
#define BUFFER_BIT_COLOR2       (1 << BUFFER_COLOR2)
#define BUFFER_BIT_COLOR3       (1 << BUFFER_COLOR3)
#define BUFFER_BIT_COLOR4       (1 << BUFFER_COLOR4)
#define BUFFER_BIT_COLOR5       (1 << BUFFER_COLOR5)
#define BUFFER_BIT_COLOR6       (1 << BUFFER_COLOR6)
#define BUFFER_BIT_COLOR7       (1 << BUFFER_COLOR7)

/**
 * Mask of all the color buffer bits (but not accum).
 */
#define BUFFER_BITS_COLOR  (BUFFER_BIT_FRONT_LEFT | \
                            BUFFER_BIT_BACK_LEFT | \
                            BUFFER_BIT_FRONT_RIGHT | \
                            BUFFER_BIT_BACK_RIGHT | \
                            BUFFER_BIT_AUX0 | \
                            BUFFER_BIT_COLOR0 | \
                            BUFFER_BIT_COLOR1 | \
                            BUFFER_BIT_COLOR2 | \
                            BUFFER_BIT_COLOR3 | \
                            BUFFER_BIT_COLOR4 | \
                            BUFFER_BIT_COLOR5 | \
                            BUFFER_BIT_COLOR6 | \
                            BUFFER_BIT_COLOR7)

/* Mask of bits for depth+stencil buffers */
#define BUFFER_BITS_DEPTH_STENCIL (BUFFER_BIT_DEPTH | BUFFER_BIT_STENCIL)

/**
 * Framebuffer configuration (aka visual / pixelformat)
 * Note: some of these fields should be boolean, but it appears that
 * code in drivers/dri/common/util.c requires int-sized fields.
 */
struct gl_config
{
   GLboolean floatMode;
   GLuint doubleBufferMode;
   GLuint stereoMode;

   GLint redBits, greenBits, blueBits, alphaBits;	/* bits per comp */
   GLuint redMask, greenMask, blueMask, alphaMask;
   GLint redShift, greenShift, blueShift, alphaShift;
   GLint rgbBits;		/* total bits for rgb */

   GLint accumRedBits, accumGreenBits, accumBlueBits, accumAlphaBits;
   GLint depthBits;
   GLint stencilBits;

   GLint numAuxBuffers;

   GLint level;

   /* EXT_visual_rating / GLX 1.2 */
   GLint visualRating;

   /* EXT_visual_info / GLX 1.2 */
   GLint transparentPixel;
   /*    colors are floats scaled to ints */
   GLint transparentRed, transparentGreen, transparentBlue, transparentAlpha;
   GLint transparentIndex;

   /* ARB_multisample / SGIS_multisample */
   GLint sampleBuffers;
   GLuint samples;

   /* SGIX_pbuffer / GLX 1.3 */
   GLint maxPbufferWidth;
   GLint maxPbufferHeight;
   GLint maxPbufferPixels;
   GLint optimalPbufferWidth;   /* Only for SGIX_pbuffer. */
   GLint optimalPbufferHeight;  /* Only for SGIX_pbuffer. */

   /* OML_swap_method */
   GLint swapMethod;

   /* EXT_texture_from_pixmap */
   GLint bindToTextureRgb;
   GLint bindToTextureRgba;
   GLint bindToMipmapTexture;
   GLint bindToTextureTargets;
   GLint yInverted;

   /* EXT_framebuffer_sRGB */
   GLint sRGBCapable;

   /* EGL_KHR_mutable_render_buffer */
   GLuint mutableRenderBuffer; /* bool */
};


/**
 * \name Bit flags used for updating material values.
 */
/*@{*/
#define MAT_ATTRIB_FRONT_AMBIENT           0
#define MAT_ATTRIB_BACK_AMBIENT            1
#define MAT_ATTRIB_FRONT_DIFFUSE           2
#define MAT_ATTRIB_BACK_DIFFUSE            3
#define MAT_ATTRIB_FRONT_SPECULAR          4
#define MAT_ATTRIB_BACK_SPECULAR           5
#define MAT_ATTRIB_FRONT_EMISSION          6
#define MAT_ATTRIB_BACK_EMISSION           7
#define MAT_ATTRIB_FRONT_SHININESS         8
#define MAT_ATTRIB_BACK_SHININESS          9
#define MAT_ATTRIB_FRONT_INDEXES           10
#define MAT_ATTRIB_BACK_INDEXES            11
#define MAT_ATTRIB_MAX                     12

#define MAT_ATTRIB_AMBIENT(f)  (MAT_ATTRIB_FRONT_AMBIENT+(f))
#define MAT_ATTRIB_DIFFUSE(f)  (MAT_ATTRIB_FRONT_DIFFUSE+(f))
#define MAT_ATTRIB_SPECULAR(f) (MAT_ATTRIB_FRONT_SPECULAR+(f))
#define MAT_ATTRIB_EMISSION(f) (MAT_ATTRIB_FRONT_EMISSION+(f))
#define MAT_ATTRIB_SHININESS(f)(MAT_ATTRIB_FRONT_SHININESS+(f))
#define MAT_ATTRIB_INDEXES(f)  (MAT_ATTRIB_FRONT_INDEXES+(f))

#define MAT_BIT_FRONT_AMBIENT         (1<<MAT_ATTRIB_FRONT_AMBIENT)
#define MAT_BIT_BACK_AMBIENT          (1<<MAT_ATTRIB_BACK_AMBIENT)
#define MAT_BIT_FRONT_DIFFUSE         (1<<MAT_ATTRIB_FRONT_DIFFUSE)
#define MAT_BIT_BACK_DIFFUSE          (1<<MAT_ATTRIB_BACK_DIFFUSE)
#define MAT_BIT_FRONT_SPECULAR        (1<<MAT_ATTRIB_FRONT_SPECULAR)
#define MAT_BIT_BACK_SPECULAR         (1<<MAT_ATTRIB_BACK_SPECULAR)
#define MAT_BIT_FRONT_EMISSION        (1<<MAT_ATTRIB_FRONT_EMISSION)
#define MAT_BIT_BACK_EMISSION         (1<<MAT_ATTRIB_BACK_EMISSION)
#define MAT_BIT_FRONT_SHININESS       (1<<MAT_ATTRIB_FRONT_SHININESS)
#define MAT_BIT_BACK_SHININESS        (1<<MAT_ATTRIB_BACK_SHININESS)
#define MAT_BIT_FRONT_INDEXES         (1<<MAT_ATTRIB_FRONT_INDEXES)
#define MAT_BIT_BACK_INDEXES          (1<<MAT_ATTRIB_BACK_INDEXES)


#define FRONT_MATERIAL_BITS   (MAT_BIT_FRONT_EMISSION | \
                               MAT_BIT_FRONT_AMBIENT | \
                               MAT_BIT_FRONT_DIFFUSE | \
                               MAT_BIT_FRONT_SPECULAR | \
                               MAT_BIT_FRONT_SHININESS | \
                               MAT_BIT_FRONT_INDEXES)

#define BACK_MATERIAL_BITS    (MAT_BIT_BACK_EMISSION | \
                               MAT_BIT_BACK_AMBIENT | \
                               MAT_BIT_BACK_DIFFUSE | \
                               MAT_BIT_BACK_SPECULAR | \
                               MAT_BIT_BACK_SHININESS | \
                               MAT_BIT_BACK_INDEXES)

#define ALL_MATERIAL_BITS     (FRONT_MATERIAL_BITS | BACK_MATERIAL_BITS)
/*@}*/


/**
 * Material state.
 */
struct gl_material
{
   GLfloat Attrib[MAT_ATTRIB_MAX][4];
};


/**
 * Light state flags.
 */
/*@{*/
#define LIGHT_SPOT         0x1
#define LIGHT_LOCAL_VIEWER 0x2
#define LIGHT_POSITIONAL   0x4
#define LIGHT_NEED_VERTICES (LIGHT_POSITIONAL|LIGHT_LOCAL_VIEWER)
/*@}*/


/**
 * Light source state.
 */
struct gl_light
{
   GLfloat Ambient[4];		/**< ambient color */
   GLfloat Diffuse[4];		/**< diffuse color */
   GLfloat Specular[4];		/**< specular color */
   GLfloat EyePosition[4];	/**< position in eye coordinates */
   GLfloat SpotDirection[4];	/**< spotlight direction in eye coordinates */
   GLfloat SpotExponent;
   GLfloat SpotCutoff;		/**< in degrees */
   GLfloat _CosCutoff;		/**< = MAX(0, cos(SpotCutoff)) */
   GLfloat ConstantAttenuation;
   GLfloat LinearAttenuation;
   GLfloat QuadraticAttenuation;
   GLboolean Enabled;		/**< On/off flag */

   /**
    * \name Derived fields
    */
   /*@{*/
   GLbitfield _Flags;		/**< Mask of LIGHT_x bits defined above */

   GLfloat _Position[4];	/**< position in eye/obj coordinates */
   GLfloat _VP_inf_norm[3];	/**< Norm direction to infinite light */
   GLfloat _h_inf_norm[3];	/**< Norm( _VP_inf_norm + <0,0,1> ) */
   GLfloat _NormSpotDirection[4]; /**< normalized spotlight direction */
   GLfloat _VP_inf_spot_attenuation;

   GLfloat _MatAmbient[2][3];	/**< material ambient * light ambient */
   GLfloat _MatDiffuse[2][3];	/**< material diffuse * light diffuse */
   GLfloat _MatSpecular[2][3];	/**< material spec * light specular */
   /*@}*/
};


/**
 * Light model state.
 */
struct gl_lightmodel
{
   GLfloat Ambient[4];		/**< ambient color */
   GLboolean LocalViewer;	/**< Local (or infinite) view point? */
   GLboolean TwoSide;		/**< Two (or one) sided lighting? */
   GLenum16 ColorControl;	/**< either GL_SINGLE_COLOR
                                     or GL_SEPARATE_SPECULAR_COLOR */
};


/**
 * Accumulation buffer attribute group (GL_ACCUM_BUFFER_BIT)
 */
struct gl_accum_attrib
{
   GLfloat ClearColor[4];	/**< Accumulation buffer clear color */
};


/**
 * Used for storing clear color, texture border color, etc.
 * The float values are typically unclamped.
 */
union gl_color_union
{
   GLfloat f[4];
   GLint i[4];
   GLuint ui[4];
};


/**
 * Color buffer attribute group (GL_COLOR_BUFFER_BIT).
 */
struct gl_colorbuffer_attrib
{
   GLuint ClearIndex;                      /**< Index for glClear */
   union gl_color_union ClearColor;        /**< Color for glClear, unclamped */
   GLuint IndexMask;                       /**< Color index write mask */

   /** 4 colormask bits per draw buffer, max 8 draw buffers. 4*8 = 32 bits */
   GLbitfield ColorMask;

   GLenum16 DrawBuffer[MAX_DRAW_BUFFERS];  /**< Which buffer to draw into */

   /**
    * \name alpha testing
    */
   /*@{*/
   GLboolean AlphaEnabled;		/**< Alpha test enabled flag */
   GLenum16 AlphaFunc;			/**< Alpha test function */
   GLfloat AlphaRefUnclamped;
   GLclampf AlphaRef;			/**< Alpha reference value */
   /*@}*/

   /**
    * \name Blending
    */
   /*@{*/
   GLbitfield BlendEnabled;		/**< Per-buffer blend enable flags */

   /* NOTE: this does _not_ depend on fragment clamping or any other clamping
    * control, only on the fixed-pointness of the render target.
    * The query does however depend on fragment color clamping.
    */
   GLfloat BlendColorUnclamped[4];      /**< Blending color */
   GLfloat BlendColor[4];		/**< Blending color */

   struct
   {
      GLenum16 SrcRGB;             /**< RGB blend source term */
      GLenum16 DstRGB;             /**< RGB blend dest term */
      GLenum16 SrcA;               /**< Alpha blend source term */
      GLenum16 DstA;               /**< Alpha blend dest term */
      GLenum16 EquationRGB;        /**< GL_ADD, GL_SUBTRACT, etc. */
      GLenum16 EquationA;          /**< GL_ADD, GL_SUBTRACT, etc. */
      /**
       * Set if any blend factor uses SRC1.  Computed at the time blend factors
       * get set.
       */
      GLboolean _UsesDualSrc;
   } Blend[MAX_DRAW_BUFFERS];
   /** Are the blend func terms currently different for each buffer/target? */
   GLboolean _BlendFuncPerBuffer;
   /** Are the blend equations currently different for each buffer/target? */
   GLboolean _BlendEquationPerBuffer;

   /**
    * Which advanced blending mode is in use (or BLEND_NONE).
    *
    * KHR_blend_equation_advanced only allows advanced blending with a single
    * draw buffer, and NVX_blend_equation_advanced_multi_draw_buffer still
    * requires all draw buffers to match, so we only need a single value.
    */
   enum gl_advanced_blend_mode _AdvancedBlendMode;

   /** Coherency requested via glEnable(GL_BLEND_ADVANCED_COHERENT_KHR)? */
   bool BlendCoherent;
   /*@}*/

   /**
    * \name Logic op
    */
   /*@{*/
   GLboolean IndexLogicOpEnabled;	/**< Color index logic op enabled flag */
   GLboolean ColorLogicOpEnabled;	/**< RGBA logic op enabled flag */
   GLenum16 LogicOp;			/**< Logic operator */
   enum gl_logicop_mode _LogicOp;
   /*@}*/

   GLboolean DitherFlag;           /**< Dither enable flag */

   GLboolean _ClampFragmentColor;  /** < with GL_FIXED_ONLY_ARB resolved */
   GLenum16 ClampFragmentColor; /**< GL_TRUE, GL_FALSE or GL_FIXED_ONLY_ARB */
   GLenum16 ClampReadColor;     /**< GL_TRUE, GL_FALSE or GL_FIXED_ONLY_ARB */

   GLboolean sRGBEnabled;  /**< Framebuffer sRGB blending/updating requested */
};


/**
 * Vertex format to describe a vertex element.
 */
struct gl_vertex_format
{
   GLenum16 Type;        /**< datatype: GL_FLOAT, GL_INT, etc */
   GLenum16 Format;      /**< default: GL_RGBA, but may be GL_BGRA */
   enum pipe_format _PipeFormat:16; /**< pipe_format for Gallium */
   GLubyte Size:5;       /**< components per element (1,2,3,4) */
   GLubyte Normalized:1; /**< GL_ARB_vertex_program */
   GLubyte Integer:1;    /**< Integer-valued? */
   GLubyte Doubles:1;    /**< double values are not converted to floats */
   GLubyte _ElementSize; /**< Size of each element in bytes */
};


/**
 * Current attribute group (GL_CURRENT_BIT).
 */
struct gl_current_attrib
{
   /**
    * \name Current vertex attributes (color, texcoords, etc).
    * \note Values are valid only after FLUSH_VERTICES has been called.
    * \note Index and Edgeflag current values are stored as floats in the
    * SIX and SEVEN attribute slots.
    * \note We need double storage for 64-bit vertex attributes
    */
   GLfloat Attrib[VERT_ATTRIB_MAX][4*2];

   /**
    * \name Current raster position attributes (always up to date after a
    * glRasterPos call).
    */
   GLfloat RasterPos[4];
   GLfloat RasterDistance;
   GLfloat RasterColor[4];
   GLfloat RasterSecondaryColor[4];
   GLfloat RasterTexCoords[MAX_TEXTURE_COORD_UNITS][4];
   GLboolean RasterPosValid;
};


/**
 * Depth buffer attribute group (GL_DEPTH_BUFFER_BIT).
 */
struct gl_depthbuffer_attrib
{
   GLenum16 Func;		/**< Function for depth buffer compare */
   GLclampd Clear;		/**< Value to clear depth buffer to */
   GLboolean Test;		/**< Depth buffering enabled flag */
   GLboolean Mask;		/**< Depth buffer writable? */
   GLboolean BoundsTest;        /**< GL_EXT_depth_bounds_test */
   GLfloat BoundsMin, BoundsMax;/**< GL_EXT_depth_bounds_test */
};


/**
 * Evaluator attribute group (GL_EVAL_BIT).
 */
struct gl_eval_attrib
{
   /**
    * \name Enable bits
    */
   /*@{*/
   GLboolean Map1Color4;
   GLboolean Map1Index;
   GLboolean Map1Normal;
   GLboolean Map1TextureCoord1;
   GLboolean Map1TextureCoord2;
   GLboolean Map1TextureCoord3;
   GLboolean Map1TextureCoord4;
   GLboolean Map1Vertex3;
   GLboolean Map1Vertex4;
   GLboolean Map2Color4;
   GLboolean Map2Index;
   GLboolean Map2Normal;
   GLboolean Map2TextureCoord1;
   GLboolean Map2TextureCoord2;
   GLboolean Map2TextureCoord3;
   GLboolean Map2TextureCoord4;
   GLboolean Map2Vertex3;
   GLboolean Map2Vertex4;
   GLboolean AutoNormal;
   /*@}*/

   /**
    * \name Map Grid endpoints and divisions and calculated du values
    */
   /*@{*/
   GLint MapGrid1un;
   GLfloat MapGrid1u1, MapGrid1u2, MapGrid1du;
   GLint MapGrid2un, MapGrid2vn;
   GLfloat MapGrid2u1, MapGrid2u2, MapGrid2du;
   GLfloat MapGrid2v1, MapGrid2v2, MapGrid2dv;
   /*@}*/
};


/**
 * Compressed fog mode.
 */
enum gl_fog_mode
{
   FOG_NONE,
   FOG_LINEAR,
   FOG_EXP,
   FOG_EXP2,
};


/**
 * Fog attribute group (GL_FOG_BIT).
 */
struct gl_fog_attrib
{
   GLboolean Enabled;		/**< Fog enabled flag */
   GLboolean ColorSumEnabled;
   uint8_t _PackedMode;		/**< Fog mode as 2 bits */
   uint8_t _PackedEnabledMode;	/**< Masked CompressedMode */
   GLfloat ColorUnclamped[4];            /**< Fog color */
   GLfloat Color[4];		/**< Fog color */
   GLfloat Density;		/**< Density >= 0.0 */
   GLfloat Start;		/**< Start distance in eye coords */
   GLfloat End;			/**< End distance in eye coords */
   GLfloat Index;		/**< Fog index */
   GLenum16 Mode;		/**< Fog mode */
   GLenum16 FogCoordinateSource;/**< GL_EXT_fog_coord */
   GLenum16 FogDistanceMode;     /**< GL_NV_fog_distance */
};


/**
 * Hint attribute group (GL_HINT_BIT).
 *
 * Values are always one of GL_FASTEST, GL_NICEST, or GL_DONT_CARE.
 */
struct gl_hint_attrib
{
   GLenum16 PerspectiveCorrection;
   GLenum16 PointSmooth;
   GLenum16 LineSmooth;
   GLenum16 PolygonSmooth;
   GLenum16 Fog;
   GLenum16 TextureCompression;   /**< GL_ARB_texture_compression */
   GLenum16 GenerateMipmap;       /**< GL_SGIS_generate_mipmap */
   GLenum16 FragmentShaderDerivative; /**< GL_ARB_fragment_shader */
   GLuint MaxShaderCompilerThreads; /**< GL_ARB_parallel_shader_compile */
};


/**
 * Lighting attribute group (GL_LIGHT_BIT).
 */
struct gl_light_attrib
{
   struct gl_light Light[MAX_LIGHTS];	/**< Array of light sources */
   struct gl_lightmodel Model;		/**< Lighting model */

   /**
    * Front and back material values.
    * Note: must call FLUSH_VERTICES() before using.
    */
   struct gl_material Material;

   GLboolean Enabled;			/**< Lighting enabled flag */
   GLboolean ColorMaterialEnabled;

   GLenum16 ShadeModel;			/**< GL_FLAT or GL_SMOOTH */
   GLenum16 ProvokingVertex;              /**< GL_EXT_provoking_vertex */
   GLenum16 ColorMaterialFace;		/**< GL_FRONT, BACK or FRONT_AND_BACK */
   GLenum16 ColorMaterialMode;		/**< GL_AMBIENT, GL_DIFFUSE, etc */
   GLbitfield _ColorMaterialBitmask;	/**< bitmask formed from Face and Mode */


   GLboolean _ClampVertexColor;
   GLenum16 ClampVertexColor;             /**< GL_TRUE, GL_FALSE, GL_FIXED_ONLY */

   /**
    * Derived state for optimizations:
    */
   /*@{*/
   GLbitfield _EnabledLights;	/**< bitmask containing enabled lights */

   GLboolean _NeedEyeCoords;
   GLboolean _NeedVertices;		/**< Use fast shader? */

   GLfloat _BaseColor[2][3];
   /*@}*/
};


/**
 * Line attribute group (GL_LINE_BIT).
 */
struct gl_line_attrib
{
   GLboolean SmoothFlag;	/**< GL_LINE_SMOOTH enabled? */
   GLboolean StippleFlag;	/**< GL_LINE_STIPPLE enabled? */
   GLushort StipplePattern;	/**< Stipple pattern */
   GLint StippleFactor;		/**< Stipple repeat factor */
   GLfloat Width;		/**< Line width */
};


/**
 * Display list attribute group (GL_LIST_BIT).
 */
struct gl_list_attrib
{
   GLuint ListBase;
};


/**
 * Multisample attribute group (GL_MULTISAMPLE_BIT).
 */
struct gl_multisample_attrib
{
   GLboolean Enabled;
   GLboolean SampleAlphaToCoverage;
   GLboolean SampleAlphaToOne;
   GLboolean SampleCoverage;
   GLboolean SampleCoverageInvert;
   GLboolean SampleShading;

   /* ARB_texture_multisample / GL3.2 additions */
   GLboolean SampleMask;

   GLfloat SampleCoverageValue;  /**< In range [0, 1] */
   GLfloat MinSampleShadingValue;  /**< In range [0, 1] */

   /** The GL spec defines this as an array but >32x MSAA is madness */
   GLbitfield SampleMaskValue;

   /* NV_alpha_to_coverage_dither_control */
   GLenum SampleAlphaToCoverageDitherControl;
};


/**
 * A pixelmap (see glPixelMap)
 */
struct gl_pixelmap
{
   GLint Size;
   GLfloat Map[MAX_PIXEL_MAP_TABLE];
};


/**
 * Collection of all pixelmaps
 */
struct gl_pixelmaps
{
   struct gl_pixelmap RtoR;  /**< i.e. GL_PIXEL_MAP_R_TO_R */
   struct gl_pixelmap GtoG;
   struct gl_pixelmap BtoB;
   struct gl_pixelmap AtoA;
   struct gl_pixelmap ItoR;
   struct gl_pixelmap ItoG;
   struct gl_pixelmap ItoB;
   struct gl_pixelmap ItoA;
   struct gl_pixelmap ItoI;
   struct gl_pixelmap StoS;
};


/**
 * Pixel attribute group (GL_PIXEL_MODE_BIT).
 */
struct gl_pixel_attrib
{
   GLenum16 ReadBuffer;		/**< source buffer for glRead/CopyPixels() */

   /*--- Begin Pixel Transfer State ---*/
   /* Fields are in the order in which they're applied... */

   /** Scale & Bias (index shift, offset) */
   /*@{*/
   GLfloat RedBias, RedScale;
   GLfloat GreenBias, GreenScale;
   GLfloat BlueBias, BlueScale;
   GLfloat AlphaBias, AlphaScale;
   GLfloat DepthBias, DepthScale;
   GLint IndexShift, IndexOffset;
   /*@}*/

   /* Pixel Maps */
   /* Note: actual pixel maps are not part of this attrib group */
   GLboolean MapColorFlag;
   GLboolean MapStencilFlag;

   /*--- End Pixel Transfer State ---*/

   /** glPixelZoom */
   GLfloat ZoomX, ZoomY;
};


/**
 * Point attribute group (GL_POINT_BIT).
 */
struct gl_point_attrib
{
   GLfloat Size;		/**< User-specified point size */
   GLfloat Params[3];		/**< GL_EXT_point_parameters */
   GLfloat MinSize, MaxSize;	/**< GL_EXT_point_parameters */
   GLfloat Threshold;		/**< GL_EXT_point_parameters */
   GLboolean SmoothFlag;	/**< True if GL_POINT_SMOOTH is enabled */
   GLboolean _Attenuated;	/**< True if Params != [1, 0, 0] */
   GLboolean PointSprite;	/**< GL_NV/ARB_point_sprite */
   GLbitfield CoordReplace;     /**< GL_ARB_point_sprite*/
   GLenum16 SpriteRMode;	/**< GL_NV_point_sprite (only!) */
   GLenum16 SpriteOrigin;	/**< GL_ARB_point_sprite */
};


/**
 * Polygon attribute group (GL_POLYGON_BIT).
 */
struct gl_polygon_attrib
{
   GLenum16 FrontFace;		/**< Either GL_CW or GL_CCW */
   GLenum FrontMode;		/**< Either GL_POINT, GL_LINE or GL_FILL */
   GLenum BackMode;		/**< Either GL_POINT, GL_LINE or GL_FILL */
   GLboolean CullFlag;		/**< Culling on/off flag */
   GLboolean SmoothFlag;	/**< True if GL_POLYGON_SMOOTH is enabled */
   GLboolean StippleFlag;	/**< True if GL_POLYGON_STIPPLE is enabled */
   GLenum16 CullFaceMode;	/**< Culling mode GL_FRONT or GL_BACK */
   GLfloat OffsetFactor;	/**< Polygon offset factor, from user */
   GLfloat OffsetUnits;		/**< Polygon offset units, from user */
   GLfloat OffsetClamp;		/**< Polygon offset clamp, from user */
   GLboolean OffsetPoint;	/**< Offset in GL_POINT mode */
   GLboolean OffsetLine;	/**< Offset in GL_LINE mode */
   GLboolean OffsetFill;	/**< Offset in GL_FILL mode */
};


/**
 * Scissor attributes (GL_SCISSOR_BIT).
 */
struct gl_scissor_rect
{
   GLint X, Y;			/**< Lower left corner of box */
   GLsizei Width, Height;	/**< Size of box */
};


struct gl_scissor_attrib
{
   GLbitfield EnableFlags;	/**< Scissor test enabled? */
   struct gl_scissor_rect ScissorArray[MAX_VIEWPORTS];
   GLint NumWindowRects;        /**< Count of enabled window rectangles */
   GLenum16 WindowRectMode;     /**< Whether to include or exclude the rects */
   struct gl_scissor_rect WindowRects[MAX_WINDOW_RECTANGLES];
};


/**
 * Stencil attribute group (GL_STENCIL_BUFFER_BIT).
 *
 * Three sets of stencil data are tracked so that OpenGL 2.0,
 * GL_EXT_stencil_two_side, and GL_ATI_separate_stencil can all be supported
 * simultaneously.  In each of the stencil state arrays, element 0 corresponds
 * to GL_FRONT.  Element 1 corresponds to the OpenGL 2.0 /
 * GL_ATI_separate_stencil GL_BACK state.  Element 2 corresponds to the
 * GL_EXT_stencil_two_side GL_BACK state.
 *
 * The derived value \c _BackFace is either 1 or 2 depending on whether or
 * not GL_STENCIL_TEST_TWO_SIDE_EXT is enabled.
 *
 * The derived value \c _TestTwoSide is set when the front-face and back-face
 * stencil state are different.
 */
struct gl_stencil_attrib
{
   GLboolean Enabled;		/**< Enabled flag */
   GLboolean TestTwoSide;	/**< GL_EXT_stencil_two_side */
   GLubyte ActiveFace;		/**< GL_EXT_stencil_two_side (0 or 2) */
   GLubyte _BackFace;           /**< Current back stencil state (1 or 2) */
   GLenum16 Function[3];	/**< Stencil function */
   GLenum16 FailFunc[3];	/**< Fail function */
   GLenum16 ZPassFunc[3];	/**< Depth buffer pass function */
   GLenum16 ZFailFunc[3];	/**< Depth buffer fail function */
   GLint Ref[3];		/**< Reference value */
   GLuint ValueMask[3];		/**< Value mask */
   GLuint WriteMask[3];		/**< Write mask */
   GLuint Clear;		/**< Clear value */
};


/**
 * Bit flags for each type of texture object
 */
/*@{*/
#define TEXTURE_2D_MULTISAMPLE_BIT (1 << TEXTURE_2D_MULTISAMPLE_INDEX)
#define TEXTURE_2D_MULTISAMPLE_ARRAY_BIT (1 << TEXTURE_2D_MULTISAMPLE_ARRAY_INDEX)
#define TEXTURE_CUBE_ARRAY_BIT (1 << TEXTURE_CUBE_ARRAY_INDEX)
#define TEXTURE_BUFFER_BIT   (1 << TEXTURE_BUFFER_INDEX)
#define TEXTURE_2D_ARRAY_BIT (1 << TEXTURE_2D_ARRAY_INDEX)
#define TEXTURE_1D_ARRAY_BIT (1 << TEXTURE_1D_ARRAY_INDEX)
#define TEXTURE_EXTERNAL_BIT (1 << TEXTURE_EXTERNAL_INDEX)
#define TEXTURE_CUBE_BIT     (1 << TEXTURE_CUBE_INDEX)
#define TEXTURE_3D_BIT       (1 << TEXTURE_3D_INDEX)
#define TEXTURE_RECT_BIT     (1 << TEXTURE_RECT_INDEX)
#define TEXTURE_2D_BIT       (1 << TEXTURE_2D_INDEX)
#define TEXTURE_1D_BIT       (1 << TEXTURE_1D_INDEX)
/*@}*/


/**
 * Texture image state.  Drivers will typically create a subclass of this
 * with extra fields for memory buffers, etc.
 */
struct gl_texture_image
{
   GLint InternalFormat;	/**< Internal format as given by the user */
   GLenum16 _BaseFormat;	/**< Either GL_RGB, GL_RGBA, GL_ALPHA,
                                 *   GL_LUMINANCE, GL_LUMINANCE_ALPHA,
                                 *   GL_INTENSITY, GL_DEPTH_COMPONENT or
                                 *   GL_DEPTH_STENCIL_EXT only. Used for
                                 *   choosing TexEnv arithmetic.
                                 */
   mesa_format TexFormat;         /**< The actual texture memory format */

   GLuint Border;		/**< 0 or 1 */
   GLuint Width;		/**< = 2^WidthLog2 + 2*Border */
   GLuint Height;		/**< = 2^HeightLog2 + 2*Border */
   GLuint Depth;		/**< = 2^DepthLog2 + 2*Border */
   GLuint Width2;		/**< = Width - 2*Border */
   GLuint Height2;		/**< = Height - 2*Border */
   GLuint Depth2;		/**< = Depth - 2*Border */
   GLuint WidthLog2;		/**< = log2(Width2) */
   GLuint HeightLog2;		/**< = log2(Height2) */
   GLuint DepthLog2;		/**< = log2(Depth2) */
   GLuint MaxNumLevels;		/**< = maximum possible number of mipmap
                                       levels, computed from the dimensions */

   struct gl_texture_object *TexObject;  /**< Pointer back to parent object */
   GLuint Level;                /**< Which mipmap level am I? */
   /** Cube map face: index into gl_texture_object::Image[] array */
   GLuint Face;

   /** GL_ARB_texture_multisample */
   GLuint NumSamples;            /**< Sample count, or 0 for non-multisample */
   GLboolean FixedSampleLocations; /**< Same sample locations for all pixels? */
};


/**
 * Indexes for cube map faces.
 */
typedef enum
{
   FACE_POS_X = 0,
   FACE_NEG_X = 1,
   FACE_POS_Y = 2,
   FACE_NEG_Y = 3,
   FACE_POS_Z = 4,
   FACE_NEG_Z = 5,
   MAX_FACES = 6
} gl_face_index;


/**
 * Sampler object state.  These objects are new with GL_ARB_sampler_objects
 * and OpenGL 3.3.  Legacy texture objects also contain a sampler object.
 */
struct gl_sampler_object
{
   simple_mtx_t Mutex;
   GLuint Name;
   GLchar *Label;               /**< GL_KHR_debug */
   GLint RefCount;

   GLenum16 WrapS;		/**< S-axis texture image wrap mode */
   GLenum16 WrapT;		/**< T-axis texture image wrap mode */
   GLenum16 WrapR;		/**< R-axis texture image wrap mode */
   GLenum16 MinFilter;		/**< minification filter */
   GLenum16 MagFilter;		/**< magnification filter */
   GLenum16 sRGBDecode;         /**< GL_DECODE_EXT or GL_SKIP_DECODE_EXT */
   union gl_color_union BorderColor;  /**< Interpreted according to texture format */
   GLfloat MinLod;		/**< min lambda, OpenGL 1.2 */
   GLfloat MaxLod;		/**< max lambda, OpenGL 1.2 */
   GLfloat LodBias;		/**< OpenGL 1.4 */
   GLfloat MaxAnisotropy;	/**< GL_EXT_texture_filter_anisotropic */
   GLenum16 CompareMode;		/**< GL_ARB_shadow */
   GLenum16 CompareFunc;		/**< GL_ARB_shadow */
   GLboolean CubeMapSeamless;   /**< GL_AMD_seamless_cubemap_per_texture */

   /** GL_ARB_bindless_texture */
   bool HandleAllocated;
   struct util_dynarray Handles;
};


/**
 * Texture object state.  Contains the array of mipmap images, border color,
 * wrap modes, filter modes, and shadow/texcompare state.
 */
struct gl_texture_object
{
   simple_mtx_t Mutex;         /**< for thread safety */
   GLint RefCount;             /**< reference count */
   GLuint Name;                /**< the user-visible texture object ID */
   GLenum16 Target;            /**< GL_TEXTURE_1D, GL_TEXTURE_2D, etc. */
   GLenum16 DepthMode;         /**< GL_ARB_depth_texture */
   GLchar *Label;              /**< GL_KHR_debug */

   struct gl_sampler_object Sampler;

   gl_texture_index TargetIndex; /**< The gl_texture_unit::CurrentTex index.
                                      Only valid when Target is valid. */
   GLfloat Priority;           /**< in [0,1] */
   GLint MaxLevel;           /**< max mipmap level (max=1000), OpenGL 1.2 */
   GLint BaseLevel;           /**< min mipmap level, OpenGL 1.2 */
   GLbyte _MaxLevel;           /**< actual max mipmap level (q in the spec) */
   GLfloat _MaxLambda;         /**< = _MaxLevel - BaseLevel (q - p in spec) */
   GLint CropRect[4];          /**< GL_OES_draw_texture */
   GLenum Swizzle[4];          /**< GL_EXT_texture_swizzle */
   GLushort _Swizzle;          /**< same as Swizzle, but SWIZZLE_* format */
   GLbyte ImmutableLevels;     /**< ES 3.0 / ARB_texture_view */
   GLboolean GenerateMipmap;   /**< GL_SGIS_generate_mipmap */
   GLboolean _BaseComplete;    /**< Is the base texture level valid? */
   GLboolean _MipmapComplete;  /**< Is the whole mipmap valid? */
   GLboolean _IsIntegerFormat; /**< Does the texture store integer values? */
   GLboolean _RenderToTexture; /**< Any rendering to this texture? */
   GLboolean Purgeable;        /**< Is the buffer purgeable under memory
                                    pressure? */
   GLboolean Immutable;        /**< GL_ARB_texture_storage */
   GLboolean _IsFloat;         /**< GL_OES_float_texture */
   GLboolean _IsHalfFloat;     /**< GL_OES_half_float_texture */
   bool StencilSampling;       /**< Should we sample stencil instead of depth? */
   bool HandleAllocated;       /**< GL_ARB_bindless_texture */

   /** GL_OES_EGL_image_external */
   GLubyte RequiredTextureImageUnits;

   GLubyte MinLevel;            /**< GL_ARB_texture_view */
   GLubyte NumLevels;           /**< GL_ARB_texture_view */
   GLushort MinLayer;            /**< GL_ARB_texture_view */
   GLushort NumLayers;           /**< GL_ARB_texture_view */

   /** GL_EXT_memory_object */
   GLenum16 TextureTiling;

   /** GL_ARB_shader_image_load_store */
   GLenum16 ImageFormatCompatibilityType;

   /** GL_ARB_texture_buffer_object */
   GLenum16 BufferObjectFormat;
   /** Equivalent Mesa format for BufferObjectFormat. */
   mesa_format _BufferObjectFormat;
   struct gl_buffer_object *BufferObject;

   /** GL_ARB_texture_buffer_range */
   GLintptr BufferOffset;
   GLsizeiptr BufferSize; /**< if this is -1, use BufferObject->Size instead */

   /** Actual texture images, indexed by [cube face] and [mipmap level] */
   struct gl_texture_image *Image[MAX_FACES][MAX_TEXTURE_LEVELS];

   /** GL_ARB_bindless_texture */
   struct util_dynarray SamplerHandles;
   struct util_dynarray ImageHandles;
};


/** Up to four combiner sources are possible with GL_NV_texture_env_combine4 */
#define MAX_COMBINER_TERMS 4


/**
 * Texture combine environment state.
 */
struct gl_tex_env_combine_state
{
   GLenum16 ModeRGB;       /**< GL_REPLACE, GL_DECAL, GL_ADD, etc. */
   GLenum16 ModeA;         /**< GL_REPLACE, GL_DECAL, GL_ADD, etc. */
   /** Source terms: GL_PRIMARY_COLOR, GL_TEXTURE, etc */
   GLenum16 SourceRGB[MAX_COMBINER_TERMS];
   GLenum16 SourceA[MAX_COMBINER_TERMS];
   /** Source operands: GL_SRC_COLOR, GL_ONE_MINUS_SRC_COLOR, etc */
   GLenum16 OperandRGB[MAX_COMBINER_TERMS];
   GLenum16 OperandA[MAX_COMBINER_TERMS];
   GLubyte ScaleShiftRGB; /**< 0, 1 or 2 */
   GLubyte ScaleShiftA;   /**< 0, 1 or 2 */
   GLubyte _NumArgsRGB;   /**< Number of inputs used for the RGB combiner */
   GLubyte _NumArgsA;     /**< Number of inputs used for the A combiner */
};


/** Compressed TexEnv effective Combine mode */
enum gl_tex_env_mode
{
   TEXENV_MODE_REPLACE,                 /* r = a0 */
   TEXENV_MODE_MODULATE,                /* r = a0 * a1 */
   TEXENV_MODE_ADD,                     /* r = a0 + a1 */
   TEXENV_MODE_ADD_SIGNED,              /* r = a0 + a1 - 0.5 */
   TEXENV_MODE_INTERPOLATE,             /* r = a0 * a2 + a1 * (1 - a2) */
   TEXENV_MODE_SUBTRACT,                /* r = a0 - a1 */
   TEXENV_MODE_DOT3_RGB,                /* r = a0 . a1 */
   TEXENV_MODE_DOT3_RGB_EXT,            /* r = a0 . a1 */
   TEXENV_MODE_DOT3_RGBA,               /* r = a0 . a1 */
   TEXENV_MODE_DOT3_RGBA_EXT,           /* r = a0 . a1 */
   TEXENV_MODE_MODULATE_ADD_ATI,        /* r = a0 * a2 + a1 */
   TEXENV_MODE_MODULATE_SIGNED_ADD_ATI, /* r = a0 * a2 + a1 - 0.5 */
   TEXENV_MODE_MODULATE_SUBTRACT_ATI,   /* r = a0 * a2 - a1 */
   TEXENV_MODE_ADD_PRODUCTS_NV,         /* r = a0 * a1 + a2 * a3 */
   TEXENV_MODE_ADD_PRODUCTS_SIGNED_NV,  /* r = a0 * a1 + a2 * a3 - 0.5 */
};


/** Compressed TexEnv Combine source */
enum gl_tex_env_source
{
   TEXENV_SRC_TEXTURE0,
   TEXENV_SRC_TEXTURE1,
   TEXENV_SRC_TEXTURE2,
   TEXENV_SRC_TEXTURE3,
   TEXENV_SRC_TEXTURE4,
   TEXENV_SRC_TEXTURE5,
   TEXENV_SRC_TEXTURE6,
   TEXENV_SRC_TEXTURE7,
   TEXENV_SRC_TEXTURE,
   TEXENV_SRC_PREVIOUS,
   TEXENV_SRC_PRIMARY_COLOR,
   TEXENV_SRC_CONSTANT,
   TEXENV_SRC_ZERO,
   TEXENV_SRC_ONE,
};


/** Compressed TexEnv Combine operand */
enum gl_tex_env_operand
{
   TEXENV_OPR_COLOR,
   TEXENV_OPR_ONE_MINUS_COLOR,
   TEXENV_OPR_ALPHA,
   TEXENV_OPR_ONE_MINUS_ALPHA,
};


/** Compressed TexEnv Combine argument */
struct gl_tex_env_argument
{
#ifdef __GNUC__
   __extension__ uint8_t Source:4;  /**< TEXENV_SRC_x */
   __extension__ uint8_t Operand:2; /**< TEXENV_OPR_x */
#else
   uint8_t Source;  /**< SRC_x */
   uint8_t Operand; /**< OPR_x */
#endif
};


/***
 * Compressed TexEnv Combine state.
 */
struct gl_tex_env_combine_packed
{
   uint32_t ModeRGB:4;        /**< Effective mode for RGB as 4 bits */
   uint32_t ModeA:4;          /**< Effective mode for RGB as 4 bits */
   uint32_t ScaleShiftRGB:2;  /**< 0, 1 or 2 */
   uint32_t ScaleShiftA:2;    /**< 0, 1 or 2 */
   uint32_t NumArgsRGB:3;     /**< Number of inputs used for the RGB combiner */
   uint32_t NumArgsA:3;       /**< Number of inputs used for the A combiner */
   /** Source arguments in a packed manner */
   struct gl_tex_env_argument ArgsRGB[MAX_COMBINER_TERMS];
   struct gl_tex_env_argument ArgsA[MAX_COMBINER_TERMS];
};


/**
 * TexGenEnabled flags.
 */
/*@{*/
#define S_BIT 1
#define T_BIT 2
#define R_BIT 4
#define Q_BIT 8
#define STR_BITS (S_BIT | T_BIT | R_BIT)
/*@}*/


/**
 * Bit flag versions of the corresponding GL_ constants.
 */
/*@{*/
#define TEXGEN_SPHERE_MAP        0x1
#define TEXGEN_OBJ_LINEAR        0x2
#define TEXGEN_EYE_LINEAR        0x4
#define TEXGEN_REFLECTION_MAP_NV 0x8
#define TEXGEN_NORMAL_MAP_NV     0x10

#define TEXGEN_NEED_NORMALS   (TEXGEN_SPHERE_MAP        | \
                               TEXGEN_REFLECTION_MAP_NV | \
                               TEXGEN_NORMAL_MAP_NV)
#define TEXGEN_NEED_EYE_COORD (TEXGEN_SPHERE_MAP        | \
                               TEXGEN_REFLECTION_MAP_NV | \
                               TEXGEN_NORMAL_MAP_NV     | \
                               TEXGEN_EYE_LINEAR)
/*@}*/



/** Tex-gen enabled for texture unit? */
#define ENABLE_TEXGEN(unit) (1 << (unit))

/** Non-identity texture matrix for texture unit? */
#define ENABLE_TEXMAT(unit) (1 << (unit))


/**
 * Texture coord generation state.
 */
struct gl_texgen
{
   GLenum16 Mode;       /**< GL_EYE_LINEAR, GL_SPHERE_MAP, etc */
   GLbitfield8 _ModeBit; /**< TEXGEN_x bit corresponding to Mode */
   GLfloat ObjectPlane[4];
   GLfloat EyePlane[4];
};


/**
 * Sampler-related subset of a texture unit, like current texture objects.
 */
struct gl_texture_unit
{
   GLfloat LodBias;		/**< for biasing mipmap levels */

   /** Texture targets that have a non-default texture bound */
   GLbitfield _BoundTextures;

   /** Current sampler object (GL_ARB_sampler_objects) */
   struct gl_sampler_object *Sampler;

   /** Current texture object pointers */
   struct gl_texture_object *CurrentTex[NUM_TEXTURE_TARGETS];

   /** Points to highest priority, complete and enabled texture object */
   struct gl_texture_object *_Current;
};


/**
 * Fixed-function-related subset of a texture unit, like enable flags,
 * texture environment/function/combiners, and texgen state.
 */
struct gl_fixedfunc_texture_unit
{
   GLbitfield16 Enabled;          /**< bitmask of TEXTURE_*_BIT flags */

   GLenum16 EnvMode;            /**< GL_MODULATE, GL_DECAL, GL_BLEND, etc. */
   GLclampf EnvColor[4];
   GLfloat EnvColorUnclamped[4];

   struct gl_texgen GenS;
   struct gl_texgen GenT;
   struct gl_texgen GenR;
   struct gl_texgen GenQ;
   GLbitfield8 TexGenEnabled;	/**< Bitwise-OR of [STRQ]_BIT values */
   GLbitfield8 _GenFlags;	/**< Bitwise-OR of Gen[STRQ]._ModeBit */

   /**
    * \name GL_EXT_texture_env_combine
    */
   struct gl_tex_env_combine_state Combine;

   /**
    * Derived state based on \c EnvMode and the \c BaseFormat of the
    * currently enabled texture.
    */
   struct gl_tex_env_combine_state _EnvMode;

   /** Current compressed TexEnv & Combine state */
   struct gl_tex_env_combine_packed _CurrentCombinePacked;

   /**
    * Currently enabled combiner state.  This will point to either
    * \c Combine or \c _EnvMode.
    */
   struct gl_tex_env_combine_state *_CurrentCombine;
};


/**
 * Texture attribute group (GL_TEXTURE_BIT).
 */
struct gl_texture_attrib
{
   struct gl_texture_object *ProxyTex[NUM_TEXTURE_TARGETS];

   /** GL_ARB_texture_buffer_object */
   struct gl_buffer_object *BufferObject;

   GLuint CurrentUnit;   /**< GL_ACTIVE_TEXTURE */

   /** Texture coord units/sets used for fragment texturing */
   GLbitfield8 _EnabledCoordUnits;

   /** Texture coord units that have texgen enabled */
   GLbitfield8 _TexGenEnabled;

   /** Texture coord units that have non-identity matrices */
   GLbitfield8 _TexMatEnabled;

   /** Bitwise-OR of all Texture.Unit[i]._GenFlags */
   GLbitfield8 _GenFlags;

   /** Largest index of a texture unit with _Current != NULL. */
   GLshort _MaxEnabledTexImageUnit;

   /** Largest index + 1 of texture units that have had any CurrentTex set. */
   GLubyte NumCurrentTexUsed;

   /** GL_ARB_seamless_cubemap */
   GLboolean CubeMapSeamless;

   struct gl_texture_unit Unit[MAX_COMBINED_TEXTURE_IMAGE_UNITS];
   struct gl_fixedfunc_texture_unit FixedFuncUnit[MAX_TEXTURE_COORD_UNITS];
};


/**
 * Data structure representing a single clip plane (e.g. one of the elements
 * of the ctx->Transform.EyeUserPlane or ctx->Transform._ClipUserPlane array).
 */
typedef GLfloat gl_clip_plane[4];


/**
 * Transformation attribute group (GL_TRANSFORM_BIT).
 */
struct gl_transform_attrib
{
   GLenum16 MatrixMode;				/**< Matrix mode */
   gl_clip_plane EyeUserPlane[MAX_CLIP_PLANES];	/**< User clip planes */
   gl_clip_plane _ClipUserPlane[MAX_CLIP_PLANES]; /**< derived */
   GLbitfield ClipPlanesEnabled;                /**< on/off bitmask */
   GLboolean Normalize;				/**< Normalize all normals? */
   GLboolean RescaleNormals;			/**< GL_EXT_rescale_normal */
   GLboolean RasterPositionUnclipped;           /**< GL_IBM_rasterpos_clip */
   GLboolean DepthClampNear;			/**< GL_AMD_depth_clamp_separate */
   GLboolean DepthClampFar;			/**< GL_AMD_depth_clamp_separate */
   /** GL_ARB_clip_control */
   GLenum16 ClipOrigin;   /**< GL_LOWER_LEFT or GL_UPPER_LEFT */
   GLenum16 ClipDepthMode;/**< GL_NEGATIVE_ONE_TO_ONE or GL_ZERO_TO_ONE */
};


/**
 * Viewport attribute group (GL_VIEWPORT_BIT).
 */
struct gl_viewport_attrib
{
   GLfloat X, Y;		/**< position */
   GLfloat Width, Height;	/**< size */
   GLfloat Near, Far;		/**< Depth buffer range */

   /**< GL_NV_viewport_swizzle */
   GLenum16 SwizzleX, SwizzleY, SwizzleZ, SwizzleW;
};


/**
 * Fields describing a mapped buffer range.
 */
struct gl_buffer_mapping
{
   GLbitfield AccessFlags; /**< Mask of GL_MAP_x_BIT flags */
   GLvoid *Pointer;        /**< User-space address of mapping */
   GLintptr Offset;        /**< Mapped offset */
   GLsizeiptr Length;      /**< Mapped length */
};


/**
 * Usages we've seen for a buffer object.
 */
typedef enum
{
   USAGE_UNIFORM_BUFFER = 0x1,
   USAGE_TEXTURE_BUFFER = 0x2,
   USAGE_ATOMIC_COUNTER_BUFFER = 0x4,
   USAGE_SHADER_STORAGE_BUFFER = 0x8,
   USAGE_TRANSFORM_FEEDBACK_BUFFER = 0x10,
   USAGE_PIXEL_PACK_BUFFER = 0x20,
   USAGE_ARRAY_BUFFER = 0x40,
   USAGE_ELEMENT_ARRAY_BUFFER = 0x80,
   USAGE_DISABLE_MINMAX_CACHE = 0x100,
} gl_buffer_usage;


/**
 * GL_ARB_vertex/pixel_buffer_object buffer object
 */
struct gl_buffer_object
{
   GLint RefCount;
   GLuint Name;
   GLchar *Label;       /**< GL_KHR_debug */
   GLenum16 Usage;      /**< GL_STREAM_DRAW_ARB, GL_STREAM_READ_ARB, etc. */
   GLbitfield StorageFlags; /**< GL_MAP_PERSISTENT_BIT, etc. */
   GLsizeiptrARB Size;  /**< Size of buffer storage in bytes */
   GLubyte *Data;       /**< Location of storage either in RAM or VRAM. */
   GLboolean DeletePending;   /**< true if buffer object is removed from the hash */
   GLboolean Written;   /**< Ever written to? (for debugging) */
   GLboolean Purgeable; /**< Is the buffer purgeable under memory pressure? */
   GLboolean Immutable; /**< GL_ARB_buffer_storage */
   gl_buffer_usage UsageHistory; /**< How has this buffer been used so far? */

   /** Counters used for buffer usage warnings */
   GLuint NumSubDataCalls;
   GLuint NumMapBufferWriteCalls;

   struct gl_buffer_mapping Mappings[MAP_COUNT];

   /** Memoization of min/max index computations for static index buffers */
   simple_mtx_t MinMaxCacheMutex;
   struct hash_table *MinMaxCache;
   unsigned MinMaxCacheHitIndices;
   unsigned MinMaxCacheMissIndices;
   bool MinMaxCacheDirty;

   bool HandleAllocated; /**< GL_ARB_bindless_texture */
};


/**
 * Client pixel packing/unpacking attributes
 */
struct gl_pixelstore_attrib
{
   GLint Alignment;
   GLint RowLength;
   GLint SkipPixels;
   GLint SkipRows;
   GLint ImageHeight;
   GLint SkipImages;
   GLboolean SwapBytes;
   GLboolean LsbFirst;
   GLboolean Invert;        /**< GL_MESA_pack_invert */
   GLint CompressedBlockWidth;   /**< GL_ARB_compressed_texture_pixel_storage */
   GLint CompressedBlockHeight;
   GLint CompressedBlockDepth;
   GLint CompressedBlockSize;
   struct gl_buffer_object *BufferObj; /**< GL_ARB_pixel_buffer_object */
};


/**
 * Enum for defining the mapping for the position/generic0 attribute.
 *
 * Do not change the order of the values as these are used as
 * array indices.
 */
typedef enum
{
   ATTRIBUTE_MAP_MODE_IDENTITY, /**< 1:1 mapping */
   ATTRIBUTE_MAP_MODE_POSITION, /**< get position and generic0 from position */
   ATTRIBUTE_MAP_MODE_GENERIC0, /**< get position and generic0 from generic0 */
   ATTRIBUTE_MAP_MODE_MAX       /**< for sizing arrays */
} gl_attribute_map_mode;


/**
 * Attributes to describe a vertex array.
 *
 * Contains the size, type, format and normalization flag,
 * along with the index of a vertex buffer binding point.
 *
 * Note that the Stride field corresponds to VERTEX_ATTRIB_ARRAY_STRIDE
 * and is only present for backwards compatibility reasons.
 * Rendering always uses VERTEX_BINDING_STRIDE.
 * The gl*Pointer() functions will set VERTEX_ATTRIB_ARRAY_STRIDE
 * and VERTEX_BINDING_STRIDE to the same value, while
 * glBindVertexBuffer() will only set VERTEX_BINDING_STRIDE.
 */
struct gl_array_attributes
{
   /** Points to client array data. Not used when a VBO is bound */
   const GLubyte *Ptr;
   /** Offset of the first element relative to the binding offset */
   GLuint RelativeOffset;
   /** Vertex format */
   struct gl_vertex_format Format;
   /** Stride as specified with gl*Pointer() */
   GLshort Stride;
   /** Index into gl_vertex_array_object::BufferBinding[] array */
   GLubyte BufferBindingIndex;

   /**
    * Derived effective buffer binding index
    *
    * Index into the gl_vertex_buffer_binding array of the vao.
    * Similar to BufferBindingIndex, but with the mapping of the
    * position/generic0 attributes applied and with identical
    * gl_vertex_buffer_binding entries collapsed to a single
    * entry within the vao.
    *
    * The value is valid past calling _mesa_update_vao_derived_arrays.
    * Note that _mesa_update_vao_derived_arrays is called when binding
    * the VAO to Array._DrawVAO.
    */
   GLubyte _EffBufferBindingIndex;
   /**
    * Derived effective relative offset.
    *
    * Relative offset to the effective buffers offset in
    * gl_vertex_buffer_binding::_EffOffset.
    *
    * The value is valid past calling _mesa_update_vao_derived_arrays.
    * Note that _mesa_update_vao_derived_arrays is called when binding
    * the VAO to Array._DrawVAO.
    */
   GLushort _EffRelativeOffset;
};


/**
 * This describes the buffer object used for a vertex array (or
 * multiple vertex arrays).  If BufferObj points to the default/null
 * buffer object, then the vertex array lives in user memory and not a VBO.
 */
struct gl_vertex_buffer_binding
{
   GLintptr Offset;                    /**< User-specified offset */
   GLsizei Stride;                     /**< User-specified stride */
   GLuint InstanceDivisor;             /**< GL_ARB_instanced_arrays */
   struct gl_buffer_object *BufferObj; /**< GL_ARB_vertex_buffer_object */
   GLbitfield _BoundArrays;            /**< Arrays bound to this binding point */

   /**
    * Derived effective bound arrays.
    *
    * The effective binding handles enabled arrays past the
    * position/generic0 attribute mapping and reduces the refered
    * gl_vertex_buffer_binding entries to a unique subset.
    *
    * The value is valid past calling _mesa_update_vao_derived_arrays.
    * Note that _mesa_update_vao_derived_arrays is called when binding
    * the VAO to Array._DrawVAO.
    */
   GLbitfield _EffBoundArrays;
   /**
    * Derived offset.
    *
    * The absolute offset to that we can collapse some attributes
    * to this unique effective binding.
    * For user space array bindings this contains the smallest pointer value
    * in the bound and interleaved arrays.
    * For VBO bindings this contains an offset that lets the attributes
    * _EffRelativeOffset stay positive and in bounds with
    * Const.MaxVertexAttribRelativeOffset
    *
    * The value is valid past calling _mesa_update_vao_derived_arrays.
    * Note that _mesa_update_vao_derived_arrays is called when binding
    * the VAO to Array._DrawVAO.
    */
   GLintptr _EffOffset;
};


/**
 * A representation of "Vertex Array Objects" (VAOs) from OpenGL 3.1+ /
 * the GL_ARB_vertex_array_object extension.
 */
struct gl_vertex_array_object
{
   /** Name of the VAO as received from glGenVertexArray. */
   GLuint Name;

   GLint RefCount;

   GLchar *Label;       /**< GL_KHR_debug */

   /**
    * Has this array object been bound?
    */
   GLboolean EverBound;

   /**
    * Marked to true if the object is shared between contexts and immutable.
    * Then reference counting is done using atomics and thread safe.
    * Is used for dlist VAOs.
    */
   bool SharedAndImmutable;

   /** Vertex attribute arrays */
   struct gl_array_attributes VertexAttrib[VERT_ATTRIB_MAX];

   /** Vertex buffer bindings */
   struct gl_vertex_buffer_binding BufferBinding[VERT_ATTRIB_MAX];

   /** Mask indicating which vertex arrays have vertex buffer associated. */
   GLbitfield VertexAttribBufferMask;

   /** Mask indicating which vertex arrays have a non-zero instance divisor. */
   GLbitfield NonZeroDivisorMask;

   /** Mask of VERT_BIT_* values indicating which arrays are enabled */
   GLbitfield Enabled;

   /**
    * Mask of VERT_BIT_* enabled arrays past position/generic0 mapping
    *
    * The value is valid past calling _mesa_update_vao_derived_arrays.
    * Note that _mesa_update_vao_derived_arrays is called when binding
    * the VAO to Array._DrawVAO.
    */
   GLbitfield _EffEnabledVBO;

   /** Same as _EffEnabledVBO, but for instance divisors. */
   GLbitfield _EffEnabledNonZeroDivisor;

   /** Denotes the way the position/generic0 attribute is mapped */
   gl_attribute_map_mode _AttributeMapMode;

   /** Mask of VERT_BIT_* values indicating changed/dirty arrays */
   GLbitfield NewArrays;

   /** The index buffer (also known as the element array buffer in OpenGL). */
   struct gl_buffer_object *IndexBufferObj;
};


/**
 * Vertex array state
 */
struct gl_array_attrib
{
   /** Currently bound array object. */
   struct gl_vertex_array_object *VAO;

   /** The default vertex array object */
   struct gl_vertex_array_object *DefaultVAO;

   /** The last VAO accessed by a DSA function */
   struct gl_vertex_array_object *LastLookedUpVAO;

   /** These contents are copied to newly created VAOs. */
   struct gl_vertex_array_object DefaultVAOState;

   /** Array objects (GL_ARB_vertex_array_object) */
   struct _mesa_HashTable *Objects;

   GLint ActiveTexture;		/**< Client Active Texture */
   GLuint LockFirst;            /**< GL_EXT_compiled_vertex_array */
   GLuint LockCount;            /**< GL_EXT_compiled_vertex_array */

   /**
    * \name Primitive restart controls
    *
    * Primitive restart is enabled if either \c PrimitiveRestart or
    * \c PrimitiveRestartFixedIndex is set.
    */
   /*@{*/
   GLboolean PrimitiveRestart;
   GLboolean PrimitiveRestartFixedIndex;
   GLboolean _PrimitiveRestart;
   GLuint RestartIndex;
   GLuint _RestartIndex[4]; /**< Restart indices for index_size - 1. */
   /*@}*/

   /* GL_ARB_vertex_buffer_object */
   struct gl_buffer_object *ArrayBufferObj;

   /**
    * Vertex array object that is used with the currently active draw command.
    * The _DrawVAO is either set to the currently bound VAO for array type
    * draws or to internal VAO's set up by the vbo module to execute immediate
    * mode or display list draws.
    */
   struct gl_vertex_array_object *_DrawVAO;
   /**
    * The VERT_BIT_* bits effectively enabled from the current _DrawVAO.
    * This is always a subset of _mesa_get_vao_vp_inputs(_DrawVAO)
    * but may omit those arrays that shall not be referenced by the current
    * gl_vertex_program_state::_VPMode. For example the generic attributes are
    * maked out form the _DrawVAO's enabled arrays when a fixed function
    * array draw is executed.
    */
   GLbitfield _DrawVAOEnabledAttribs;
   /**
    * Initially or if the VAO referenced by _DrawVAO is deleted the _DrawVAO
    * pointer is set to the _EmptyVAO which is just an empty VAO all the time.
    */
   struct gl_vertex_array_object *_EmptyVAO;

   /** Legal array datatypes and the API for which they have been computed */
   GLbitfield LegalTypesMask;
   gl_api LegalTypesMaskAPI;
};


/**
 * Feedback buffer state
 */
struct gl_feedback
{
   GLenum16 Type;
   GLbitfield _Mask;    /**< FB_* bits */
   GLfloat *Buffer;
   GLuint BufferSize;
   GLuint Count;
};


/**
 * Selection buffer state
 */
struct gl_selection
{
   GLuint *Buffer;	/**< selection buffer */
   GLuint BufferSize;	/**< size of the selection buffer */
   GLuint BufferCount;	/**< number of values in the selection buffer */
   GLuint Hits;		/**< number of records in the selection buffer */
   GLuint NameStackDepth; /**< name stack depth */
   GLuint NameStack[MAX_NAME_STACK_DEPTH]; /**< name stack */
   GLboolean HitFlag;	/**< hit flag */
   GLfloat HitMinZ;	/**< minimum hit depth */
   GLfloat HitMaxZ;	/**< maximum hit depth */
};


/**
 * 1-D Evaluator control points
 */
struct gl_1d_map
{
   GLuint Order;	/**< Number of control points */
   GLfloat u1, u2, du;	/**< u1, u2, 1.0/(u2-u1) */
   GLfloat *Points;	/**< Points to contiguous control points */
};


/**
 * 2-D Evaluator control points
 */
struct gl_2d_map
{
   GLuint Uorder;		/**< Number of control points in U dimension */
   GLuint Vorder;		/**< Number of control points in V dimension */
   GLfloat u1, u2, du;
   GLfloat v1, v2, dv;
   GLfloat *Points;		/**< Points to contiguous control points */
};


/**
 * All evaluator control point state
 */
struct gl_evaluators
{
   /**
    * \name 1-D maps
    */
   /*@{*/
   struct gl_1d_map Map1Vertex3;
   struct gl_1d_map Map1Vertex4;
   struct gl_1d_map Map1Index;
   struct gl_1d_map Map1Color4;
   struct gl_1d_map Map1Normal;
   struct gl_1d_map Map1Texture1;
   struct gl_1d_map Map1Texture2;
   struct gl_1d_map Map1Texture3;
   struct gl_1d_map Map1Texture4;
   /*@}*/

   /**
    * \name 2-D maps
    */
   /*@{*/
   struct gl_2d_map Map2Vertex3;
   struct gl_2d_map Map2Vertex4;
   struct gl_2d_map Map2Index;
   struct gl_2d_map Map2Color4;
   struct gl_2d_map Map2Normal;
   struct gl_2d_map Map2Texture1;
   struct gl_2d_map Map2Texture2;
   struct gl_2d_map Map2Texture3;
   struct gl_2d_map Map2Texture4;
   /*@}*/
};


struct gl_transform_feedback_varying_info
{
   char *Name;
   GLenum16 Type;
   GLint BufferIndex;
   GLint Size;
   GLint Offset;
};


/**
 * Per-output info vertex shaders for transform feedback.
 */
struct gl_transform_feedback_output
{
   uint32_t OutputRegister;
   uint32_t OutputBuffer;
   uint32_t NumComponents;
   uint32_t StreamId;

   /** offset (in DWORDs) of this output within the interleaved structure */
   uint32_t DstOffset;

   /**
    * Offset into the output register of the data to output.  For example,
    * if NumComponents is 2 and ComponentOffset is 1, then the data to
    * offset is in the y and z components of the output register.
    */
   uint32_t ComponentOffset;
};


struct gl_transform_feedback_buffer
{
   uint32_t Binding;

   uint32_t NumVaryings;

   /**
    * Total number of components stored in each buffer.  This may be used by
    * hardware back-ends to determine the correct stride when interleaving
    * multiple transform feedback outputs in the same buffer.
    */
   uint32_t Stride;

   /**
    * Which transform feedback stream this buffer binding is associated with.
    */
   uint32_t Stream;
};


/** Post-link transform feedback info. */
struct gl_transform_feedback_info
{
   unsigned NumOutputs;

   /* Bitmask of active buffer indices. */
   unsigned ActiveBuffers;

   struct gl_transform_feedback_output *Outputs;

   /** Transform feedback varyings used for the linking of this shader program.
    *
    * Use for glGetTransformFeedbackVarying().
    */
   struct gl_transform_feedback_varying_info *Varyings;
   GLint NumVarying;

   struct gl_transform_feedback_buffer Buffers[MAX_FEEDBACK_BUFFERS];
};


/**
 * Transform feedback object state
 */
struct gl_transform_feedback_object
{
   GLuint Name;  /**< AKA the object ID */
   GLint RefCount;
   GLchar *Label;     /**< GL_KHR_debug */
   GLboolean Active;  /**< Is transform feedback enabled? */
   GLboolean Paused;  /**< Is transform feedback paused? */
   GLboolean EndedAnytime; /**< Has EndTransformFeedback been called
                                at least once? */
   GLboolean EverBound; /**< Has this object been bound? */

   /**
    * GLES: if Active is true, remaining number of primitives which can be
    * rendered without overflow.  This is necessary to track because GLES
    * requires us to generate INVALID_OPERATION if a call to glDrawArrays or
    * glDrawArraysInstanced would overflow transform feedback buffers.
    * Undefined if Active is false.
    *
    * Not tracked for desktop GL since it's unnecessary.
    */
   unsigned GlesRemainingPrims;

   /**
    * The program active when BeginTransformFeedback() was called.
    * When active and unpaused, this equals ctx->Shader.CurrentProgram[stage],
    * where stage is the pipeline stage that is the source of data for
    * transform feedback.
    */
   struct gl_program *program;

   /** The feedback buffers */
   GLuint BufferNames[MAX_FEEDBACK_BUFFERS];
   struct gl_buffer_object *Buffers[MAX_FEEDBACK_BUFFERS];

   /** Start of feedback data in dest buffer */
   GLintptr Offset[MAX_FEEDBACK_BUFFERS];

   /**
    * Max data to put into dest buffer (in bytes).  Computed based on
    * RequestedSize and the actual size of the buffer.
    */
   GLsizeiptr Size[MAX_FEEDBACK_BUFFERS];

   /**
    * Size that was specified when the buffer was bound.  If the buffer was
    * bound with glBindBufferBase() or glBindBufferOffsetEXT(), this value is
    * zero.
    */
   GLsizeiptr RequestedSize[MAX_FEEDBACK_BUFFERS];
};


/**
 * Context state for transform feedback.
 */
struct gl_transform_feedback_state
{
   GLenum16 Mode;     /**< GL_POINTS, GL_LINES or GL_TRIANGLES */

   /** The general binding point (GL_TRANSFORM_FEEDBACK_BUFFER) */
   struct gl_buffer_object *CurrentBuffer;

   /** The table of all transform feedback objects */
   struct _mesa_HashTable *Objects;

   /** The current xform-fb object (GL_TRANSFORM_FEEDBACK_BINDING) */
   struct gl_transform_feedback_object *CurrentObject;

   /** The default xform-fb object (Name==0) */
   struct gl_transform_feedback_object *DefaultObject;
};


/**
 * A "performance monitor" as described in AMD_performance_monitor.
 */
struct gl_perf_monitor_object
{
   GLuint Name;

   /** True if the monitor is currently active (Begin called but not End). */
   GLboolean Active;

   /**
    * True if the monitor has ended.
    *
    * This is distinct from !Active because it may never have began.
    */
   GLboolean Ended;

   /**
    * A list of groups with currently active counters.
    *
    * ActiveGroups[g] == n if there are n counters active from group 'g'.
    */
   unsigned *ActiveGroups;

   /**
    * An array of bitsets, subscripted by group ID, then indexed by counter ID.
    *
    * Checking whether counter 'c' in group 'g' is active can be done via:
    *
    *    BITSET_TEST(ActiveCounters[g], c)
    */
   GLuint **ActiveCounters;
};


union gl_perf_monitor_counter_value
{
   float f;
   uint64_t u64;
   uint32_t u32;
};


struct gl_perf_monitor_counter
{
   /** Human readable name for the counter. */
   const char *Name;

   /**
    * Data type of the counter.  Valid values are FLOAT, UNSIGNED_INT,
    * UNSIGNED_INT64_AMD, and PERCENTAGE_AMD.
    */
   GLenum16 Type;

   /** Minimum counter value. */
   union gl_perf_monitor_counter_value Minimum;

   /** Maximum counter value. */
   union gl_perf_monitor_counter_value Maximum;
};


struct gl_perf_monitor_group
{
   /** Human readable name for the group. */
   const char *Name;

   /**
    * Maximum number of counters in this group which can be active at the
    * same time.
    */
   GLuint MaxActiveCounters;

   /** Array of counters within this group. */
   const struct gl_perf_monitor_counter *Counters;
   GLuint NumCounters;
};


/**
 * A query object instance as described in INTEL_performance_query.
 *
 * NB: We want to keep this and the corresponding backend structure
 * relatively lean considering that applications may expect to
 * allocate enough objects to be able to query around all draw calls
 * in a frame.
 */
struct gl_perf_query_object
{
   GLuint Id;          /**< hash table ID/name */
   unsigned Used:1;    /**< has been used for 1 or more queries */
   unsigned Active:1;  /**< inside Begin/EndPerfQuery */
   unsigned Ready:1;   /**< result is ready? */
};


/**
 * Context state for AMD_performance_monitor.
 */
struct gl_perf_monitor_state
{
   /** Array of performance monitor groups (indexed by group ID) */
   const struct gl_perf_monitor_group *Groups;
   GLuint NumGroups;

   /** The table of all performance monitors. */
   struct _mesa_HashTable *Monitors;
};


/**
 * Context state for INTEL_performance_query.
 */
struct gl_perf_query_state
{
   struct _mesa_HashTable *Objects; /**< The table of all performance query objects */
};


/**
 * A bindless sampler object.
 */
struct gl_bindless_sampler
{
   /** Texture unit (set by glUniform1()). */
   GLubyte unit;

   /** Whether this bindless sampler is bound to a unit. */
   GLboolean bound;

   /** Texture Target (TEXTURE_1D/2D/3D/etc_INDEX). */
   gl_texture_index target;

   /** Pointer to the base of the data. */
   GLvoid *data;
};


/**
 * A bindless image object.
 */
struct gl_bindless_image
{
   /** Image unit (set by glUniform1()). */
   GLubyte unit;

   /** Whether this bindless image is bound to a unit. */
   GLboolean bound;

   /** Access qualifier (GL_READ_WRITE, GL_READ_ONLY, GL_WRITE_ONLY, or
    * GL_NONE to indicate both read-only and write-only)
    */
   GLenum16 access;

   /** Pointer to the base of the data. */
   GLvoid *data;
};


/**
 * Current vertex processing mode: fixed function vs. shader.
 * In reality, fixed function is probably implemented by a shader but that's
 * not what we care about here.
 */
typedef enum
{
   VP_MODE_FF,     /**< legacy / fixed function */
   VP_MODE_SHADER, /**< ARB vertex program or GLSL vertex shader */
   VP_MODE_MAX     /**< for sizing arrays */
} gl_vertex_processing_mode;


/**
 * Base class for any kind of program object
 */
struct gl_program
{
   /** FIXME: This must be first until we split shader_info from nir_shader */
   struct shader_info info;

   GLuint Id;
   GLint RefCount;
   GLubyte *String;  /**< Null-terminated program text */

   /** GL_VERTEX/FRAGMENT_PROGRAM_ARB, GL_GEOMETRY_PROGRAM_NV */
   GLenum16 Target;
   GLenum16 Format;    /**< String encoding format */

   GLboolean _Used;        /**< Ever used for drawing? Used for debugging */

   struct nir_shader *nir;

   /* Saved and restored with metadata. Freed with ralloc. */
   void *driver_cache_blob;
   size_t driver_cache_blob_size;

   bool is_arb_asm; /** Is this an ARB assembly-style program */

   /** Is this program written to on disk shader cache */
   bool program_written_to_cache;

   /** A bitfield indicating which vertex shader inputs consume two slots
    *
    * This is used for mapping from single-slot input locations in the GL API
    * to dual-slot double input locations in the shader.  This field is set
    * once as part of linking and never updated again to ensure the mapping
    * remains consistent.
    *
    * Note: There may be dual-slot variables in the original shader source
    * which do not appear in this bitfield due to having been eliminated by
    * the compiler prior to DualSlotInputs being calculated.  There may also
    * be bits set in this bitfield which are set but which the shader never
    * reads due to compiler optimizations eliminating such variables after
    * DualSlotInputs is calculated.
    */
   GLbitfield64 DualSlotInputs;
   /** Subset of OutputsWritten outputs written with non-zero index. */
   GLbitfield64 SecondaryOutputsWritten;
   /** TEXTURE_x_BIT bitmask */
   GLbitfield16 TexturesUsed[MAX_COMBINED_TEXTURE_IMAGE_UNITS];
   /** Bitfield of which samplers are used */
   GLbitfield SamplersUsed;
   /** Texture units used for shadow sampling. */
   GLbitfield ShadowSamplers;
   /** Texture units used for samplerExternalOES */
   GLbitfield ExternalSamplersUsed;

   /** Named parameters, constants, etc. from program text */
   struct gl_program_parameter_list *Parameters;

   /** Map from sampler unit to texture unit (set by glUniform1i()) */
   GLubyte SamplerUnits[MAX_SAMPLERS];

   /* FIXME: We should be able to make this struct a union. However some
    * drivers (i915/fragment_programs, swrast/prog_execute) mix the use of
    * these fields, we should fix this.
    */
   struct {
      /** Fields used by GLSL programs */
      struct {
         /** Data shared by gl_program and gl_shader_program */
         struct gl_shader_program_data *data;

         struct gl_active_atomic_buffer **AtomicBuffers;

         /** Post-link transform feedback info. */
         struct gl_transform_feedback_info *LinkedTransformFeedback;

         /**
          * Number of types for subroutine uniforms.
          */
         GLuint NumSubroutineUniformTypes;

         /**
          * Subroutine uniform remap table
          * based on the program level uniform remap table.
          */
         GLuint NumSubroutineUniforms; /* non-sparse total */
         GLuint NumSubroutineUniformRemapTable;
         struct gl_uniform_storage **SubroutineUniformRemapTable;

         /**
          * Num of subroutine functions for this stage and storage for them.
          */
         GLuint NumSubroutineFunctions;
         GLuint MaxSubroutineFunctionIndex;
         struct gl_subroutine_function *SubroutineFunctions;

         /**
          * Map from image uniform index to image unit (set by glUniform1i())
          *
          * An image uniform index is associated with each image uniform by
          * the linker.  The image index associated with each uniform is
          * stored in the \c gl_uniform_storage::image field.
          */
         GLubyte ImageUnits[MAX_IMAGE_UNIFORMS];

         /**
          * Access qualifier specified in the shader for each image uniform
          * index.  Either \c GL_READ_ONLY, \c GL_WRITE_ONLY, \c
          * GL_READ_WRITE, or \c GL_NONE to indicate both read-only and
          * write-only.
          *
          * It may be different, though only more strict than the value of
          * \c gl_image_unit::Access for the corresponding image unit.
          */
         GLenum16 ImageAccess[MAX_IMAGE_UNIFORMS];

         struct gl_uniform_block **UniformBlocks;
         struct gl_uniform_block **ShaderStorageBlocks;

         /**
          * Bitmask of shader storage blocks not declared as read-only.
          */
         unsigned ShaderStorageBlocksWriteAccess;

         /** Which texture target is being sampled
          * (TEXTURE_1D/2D/3D/etc_INDEX)
          */
         GLubyte SamplerTargets[MAX_SAMPLERS];

         /**
          * Number of samplers declared with the bindless_sampler layout
          * qualifier as specified by ARB_bindless_texture.
          */
         GLuint NumBindlessSamplers;
         GLboolean HasBoundBindlessSampler;
         struct gl_bindless_sampler *BindlessSamplers;

         /**
          * Number of images declared with the bindless_image layout qualifier
          * as specified by ARB_bindless_texture.
          */
         GLuint NumBindlessImages;
         GLboolean HasBoundBindlessImage;
         struct gl_bindless_image *BindlessImages;

         union {
            struct {
               /**
                * A bitmask of gl_advanced_blend_mode values
                */
               GLbitfield BlendSupport;
            } fs;
         };
      } sh;

      /** ARB assembly-style program fields */
      struct {
         struct prog_instruction *Instructions;

         /**
          * Local parameters used by the program.
          *
          * It's dynamically allocated because it is rarely used (just
          * assembly-style programs), and MAX_PROGRAM_LOCAL_PARAMS entries
          * once it's allocated.
          */
         GLfloat (*LocalParams)[4];

         /** Bitmask of which register files are read/written with indirect
          * addressing.  Mask of (1 << PROGRAM_x) bits.
          */
         GLbitfield IndirectRegisterFiles;

         /** Logical counts */
         /*@{*/
         GLuint NumInstructions;
         GLuint NumTemporaries;
         GLuint NumParameters;
         GLuint NumAttributes;
         GLuint NumAddressRegs;
         GLuint NumAluInstructions;
         GLuint NumTexInstructions;
         GLuint NumTexIndirections;
         /*@}*/
         /** Native, actual h/w counts */
         /*@{*/
         GLuint NumNativeInstructions;
         GLuint NumNativeTemporaries;
         GLuint NumNativeParameters;
         GLuint NumNativeAttributes;
         GLuint NumNativeAddressRegs;
         GLuint NumNativeAluInstructions;
         GLuint NumNativeTexInstructions;
         GLuint NumNativeTexIndirections;
         /*@}*/

         /** Used by ARB assembly-style programs. Can only be true for vertex
          * programs.
          */
         GLboolean IsPositionInvariant;
      } arb;
   };
};


/**
 * State common to vertex and fragment programs.
 */
struct gl_program_state
{
   GLint ErrorPos;                       /* GL_PROGRAM_ERROR_POSITION_ARB/NV */
   const char *ErrorString;              /* GL_PROGRAM_ERROR_STRING_ARB/NV */
};


/**
 * Context state for vertex programs.
 */
struct gl_vertex_program_state
{
   GLboolean Enabled;            /**< User-set GL_VERTEX_PROGRAM_ARB/NV flag */
   GLboolean PointSizeEnabled;   /**< GL_VERTEX_PROGRAM_POINT_SIZE_ARB/NV */
   GLboolean TwoSideEnabled;     /**< GL_VERTEX_PROGRAM_TWO_SIDE_ARB/NV */
   /** Should fixed-function T&L be implemented with a vertex prog? */
   GLboolean _MaintainTnlProgram;

   struct gl_program *Current;  /**< User-bound vertex program */

   /** Currently enabled and valid vertex program (including internal
    * programs, user-defined vertex programs and GLSL vertex shaders).
    * This is the program we must use when rendering.
    */
   struct gl_program *_Current;

   GLfloat Parameters[MAX_PROGRAM_ENV_PARAMS][4]; /**< Env params */

   /** Program to emulate fixed-function T&L (see above) */
   struct gl_program *_TnlProgram;

   /** Cache of fixed-function programs */
   struct gl_program_cache *Cache;

   GLboolean _Overriden;

   /**
    * If we have a vertex program, a TNL program or no program at all.
    * Note that this value should be kept up to date all the time,
    * nevertheless its correctness is asserted in _mesa_update_state.
    * The reason is to avoid calling _mesa_update_state twice we need
    * this value on draw *before* actually calling _mesa_update_state.
    * Also it should need to get recomputed only on changes to the
    * vertex program which are heavyweight already.
    */
   gl_vertex_processing_mode _VPMode;
};

/**
 * Context state for tessellation control programs.
 */
struct gl_tess_ctrl_program_state
{
   /** Currently bound and valid shader. */
   struct gl_program *_Current;

   GLint patch_vertices;
   GLfloat patch_default_outer_level[4];
   GLfloat patch_default_inner_level[2];
};

/**
 * Context state for tessellation evaluation programs.
 */
struct gl_tess_eval_program_state
{
   /** Currently bound and valid shader. */
   struct gl_program *_Current;
};

/**
 * Context state for geometry programs.
 */
struct gl_geometry_program_state
{
   /**
    * Currently enabled and valid program (including internal programs
    * and compiled shader programs).
    */
   struct gl_program *_Current;
};

/**
 * Context state for fragment programs.
 */
struct gl_fragment_program_state
{
   GLboolean Enabled;     /**< User-set fragment program enable flag */
   /** Should fixed-function texturing be implemented with a fragment prog? */
   GLboolean _MaintainTexEnvProgram;

   struct gl_program *Current;  /**< User-bound fragment program */

   /**
    * Currently enabled and valid fragment program (including internal
    * programs, user-defined fragment programs and GLSL fragment shaders).
    * This is the program we must use when rendering.
    */
   struct gl_program *_Current;

   GLfloat Parameters[MAX_PROGRAM_ENV_PARAMS][4]; /**< Env params */

   /** Program to emulate fixed-function texture env/combine (see above) */
   struct gl_program *_TexEnvProgram;

   /** Cache of fixed-function programs */
   struct gl_program_cache *Cache;
};


/**
 * Context state for compute programs.
 */
struct gl_compute_program_state
{
   /** Currently enabled and valid program (including internal programs
    * and compiled shader programs).
    */
   struct gl_program *_Current;
};


/**
 * ATI_fragment_shader runtime state
 */

struct atifs_instruction;
struct atifs_setupinst;

/**
 * ATI fragment shader
 */
struct ati_fragment_shader
{
   GLuint Id;
   GLint RefCount;
   struct atifs_instruction *Instructions[2];
   struct atifs_setupinst *SetupInst[2];
   GLfloat Constants[8][4];
   GLbitfield LocalConstDef;  /**< Indicates which constants have been set */
   GLubyte numArithInstr[2];
   GLubyte regsAssigned[2];
   GLubyte NumPasses;         /**< 1 or 2 */
   /**
    * Current compile stage: 0 setup pass1, 1 arith pass1,
    * 2 setup pass2, 3 arith pass2.
    */
   GLubyte cur_pass;
   GLubyte last_optype;
   GLboolean interpinp1;
   GLboolean isValid;
   /**
    * Array of 2 bit values for each tex unit to remember whether
    * STR or STQ swizzle was used
    */
   GLuint swizzlerq;
   struct gl_program *Program;
};

/**
 * Context state for GL_ATI_fragment_shader
 */
struct gl_ati_fragment_shader_state
{
   GLboolean Enabled;
   GLboolean Compiling;
   GLfloat GlobalConstants[8][4];
   struct ati_fragment_shader *Current;
};

/**
 *  Shader subroutine function definition
 */
struct gl_subroutine_function
{
   char *name;
   int index;
   int num_compat_types;
   const struct glsl_type **types;
};

/**
 * Shader information needed by both gl_shader and gl_linked shader.
 */
struct gl_shader_info
{
   /**
    * Tessellation Control shader state from layout qualifiers.
    */
   struct {
      /**
       * 0 - vertices not declared in shader, or
       * 1 .. GL_MAX_PATCH_VERTICES
       */
      GLint VerticesOut;
   } TessCtrl;

   /**
    * Tessellation Evaluation shader state from layout qualifiers.
    */
   struct {
      /**
       * GL_TRIANGLES, GL_QUADS, GL_ISOLINES or PRIM_UNKNOWN if it's not set
       * in this shader.
       */
      GLenum16 PrimitiveMode;

      enum gl_tess_spacing Spacing;

      /**
       * GL_CW, GL_CCW, or 0 if it's not set in this shader.
       */
      GLenum16 VertexOrder;
      /**
       * 1, 0, or -1 if it's not set in this shader.
       */
      int PointMode;
   } TessEval;

   /**
    * Geometry shader state from GLSL 1.50 layout qualifiers.
    */
   struct {
      GLint VerticesOut;
      /**
       * 0 - Invocations count not declared in shader, or
       * 1 .. Const.MaxGeometryShaderInvocations
       */
      GLint Invocations;
      /**
       * GL_POINTS, GL_LINES, GL_LINES_ADJACENCY, GL_TRIANGLES, or
       * GL_TRIANGLES_ADJACENCY, or PRIM_UNKNOWN if it's not set in this
       * shader.
       */
      GLenum16 InputType;
       /**
        * GL_POINTS, GL_LINE_STRIP or GL_TRIANGLE_STRIP, or PRIM_UNKNOWN if
        * it's not set in this shader.
        */
      GLenum16 OutputType;
   } Geom;

   /**
    * Compute shader state from ARB_compute_shader and
    * ARB_compute_variable_group_size layout qualifiers.
    */
   struct {
      /**
       * Size specified using local_size_{x,y,z}, or all 0's to indicate that
       * it's not set in this shader.
       */
      unsigned LocalSize[3];

      /**
       * Whether a variable work group size has been specified as defined by
       * ARB_compute_variable_group_size.
       */
      bool LocalSizeVariable;

      /*
       * Arrangement of invocations used to calculate derivatives in a compute
       * shader.  From NV_compute_shader_derivatives.
       */
      enum gl_derivative_group DerivativeGroup;
   } Comp;
};

/**
 * A linked GLSL shader object.
 */
struct gl_linked_shader
{
   gl_shader_stage Stage;

#ifdef DEBUG
   unsigned SourceChecksum;
#endif

   struct gl_program *Program;  /**< Post-compile assembly code */

   /**
    * \name Sampler tracking
    *
    * \note Each of these fields is only set post-linking.
    */
   /*@{*/
   GLbitfield shadow_samplers;	/**< Samplers used for shadow sampling. */
   /*@}*/

   /**
    * Number of default uniform block components used by this shader.
    *
    * This field is only set post-linking.
    */
   unsigned num_uniform_components;

   /**
    * Number of combined uniform components used by this shader.
    *
    * This field is only set post-linking.  It is the sum of the uniform block
    * sizes divided by sizeof(float), and num_uniform_compoennts.
    */
   unsigned num_combined_uniform_components;

   struct exec_list *ir;
   struct exec_list *packed_varyings;
   struct exec_list *fragdata_arrays;
   struct glsl_symbol_table *symbols;

   /**
    * ARB_gl_spirv related data.
    *
    * This is actually a reference to the gl_shader::spirv_data, which
    * stores information that is also needed during linking.
    */
   struct gl_shader_spirv_data *spirv_data;
};


/**
 * Compile status enum. COMPILE_SKIPPED is used to indicate the compile
 * was skipped due to the shader matching one that's been seen before by
 * the on-disk cache.
 */
enum gl_compile_status
{
   COMPILE_FAILURE = 0,
   COMPILE_SUCCESS,
   COMPILE_SKIPPED
};

/**
 * A GLSL shader object.
 */
struct gl_shader
{
   /** GL_FRAGMENT_SHADER || GL_VERTEX_SHADER || GL_GEOMETRY_SHADER_ARB ||
    *  GL_TESS_CONTROL_SHADER || GL_TESS_EVALUATION_SHADER.
    * Must be the first field.
    */
   GLenum16 Type;
   gl_shader_stage Stage;
   GLuint Name;  /**< AKA the handle */
   GLint RefCount;  /**< Reference count */
   GLchar *Label;   /**< GL_KHR_debug */
   unsigned char sha1[20]; /**< SHA1 hash of pre-processed source */
   GLboolean DeletePending;
   bool IsES;              /**< True if this shader uses GLSL ES */

   enum gl_compile_status CompileStatus;

#ifdef DEBUG
   unsigned SourceChecksum;       /**< for debug/logging purposes */
#endif
   const GLchar *Source;  /**< Source code string */

   const GLchar *FallbackSource;  /**< Fallback string used by on-disk cache*/

   GLchar *InfoLog;

   unsigned Version;       /**< GLSL version used for linking */

   /**
    * A bitmask of gl_advanced_blend_mode values
    */
   GLbitfield BlendSupport;

   struct exec_list *ir;
   struct glsl_symbol_table *symbols;

   /**
    * Whether early fragment tests are enabled as defined by
    * ARB_shader_image_load_store.
    */
   bool EarlyFragmentTests;

   bool ARB_fragment_coord_conventions_enable;

   bool redeclares_gl_fragcoord;
   bool uses_gl_fragcoord;

   bool PostDepthCoverage;
   bool PixelInterlockOrdered;
   bool PixelInterlockUnordered;
   bool SampleInterlockOrdered;
   bool SampleInterlockUnordered;
   bool InnerCoverage;

   /**
    * Fragment shader state from GLSL 1.50 layout qualifiers.
    */
   bool origin_upper_left;
   bool pixel_center_integer;

   /**
    * Whether bindless_sampler/bindless_image, and respectively
    * bound_sampler/bound_image are declared at global scope as defined by
    * ARB_bindless_texture.
    */
   bool bindless_sampler;
   bool bindless_image;
   bool bound_sampler;
   bool bound_image;

   /**
    * Whether layer output is viewport-relative.
    */
   bool redeclares_gl_layer;
   bool layer_viewport_relative;

   /** Global xfb_stride out qualifier if any */
   GLuint TransformFeedbackBufferStride[MAX_FEEDBACK_BUFFERS];

   struct gl_shader_info info;

   /* ARB_gl_spirv related data */
   struct gl_shader_spirv_data *spirv_data;
};


struct gl_uniform_buffer_variable
{
   char *Name;

   /**
    * Name of the uniform as seen by glGetUniformIndices.
    *
    * glGetUniformIndices requires that the block instance index \b not be
    * present in the name of queried uniforms.
    *
    * \note
    * \c gl_uniform_buffer_variable::IndexName and
    * \c gl_uniform_buffer_variable::Name may point to identical storage.
    */
   char *IndexName;

   const struct glsl_type *Type;
   unsigned int Offset;
   GLboolean RowMajor;
};


struct gl_uniform_block
{
   /** Declared name of the uniform block */
   char *Name;

   /** Array of supplemental information about UBO ir_variables. */
   struct gl_uniform_buffer_variable *Uniforms;
   GLuint NumUniforms;

   /**
    * Index (GL_UNIFORM_BLOCK_BINDING) into ctx->UniformBufferBindings[] to use
    * with glBindBufferBase to bind a buffer object to this uniform block.
    */
   GLuint Binding;

   /**
    * Minimum size (in bytes) of a buffer object to back this uniform buffer
    * (GL_UNIFORM_BLOCK_DATA_SIZE).
    */
   GLuint UniformBufferSize;

   /** Stages that reference this block */
   uint8_t stageref;

   /**
    * Linearized array index for uniform block instance arrays
    *
    * Given a uniform block instance array declared with size
    * blk[s_0][s_1]..[s_m], the block referenced by blk[i_0][i_1]..[i_m] will
    * have the linearized array index
    *
    *           m-1       m
    *     i_m + ∑   i_j * ∏     s_k
    *           j=0       k=j+1
    *
    * For a uniform block instance that is not an array, this is always 0.
    */
   uint8_t linearized_array_index;

   /**
    * Layout specified in the shader
    *
    * This isn't accessible through the API, but it is used while
    * cross-validating uniform blocks.
    */
   enum glsl_interface_packing _Packing;
   GLboolean _RowMajor;
};

/**
 * Structure that represents a reference to an atomic buffer from some
 * shader program.
 */
struct gl_active_atomic_buffer
{
   /** Uniform indices of the atomic counters declared within it. */
   GLuint *Uniforms;
   GLuint NumUniforms;

   /** Binding point index associated with it. */
   GLuint Binding;

   /** Minimum reasonable size it is expected to have. */
   GLuint MinimumSize;

   /** Shader stages making use of it. */
   GLboolean StageReferences[MESA_SHADER_STAGES];
};

/**
 * Data container for shader queries. This holds only the minimal
 * amount of required information for resource queries to work.
 */
struct gl_shader_variable
{
   /**
    * Declared type of the variable
    */
   const struct glsl_type *type;

   /**
    * If the variable is in an interface block, this is the type of the block.
    */
   const struct glsl_type *interface_type;

   /**
    * For variables inside structs (possibly recursively), this is the
    * outermost struct type.
    */
   const struct glsl_type *outermost_struct_type;

   /**
    * Declared name of the variable
    */
   char *name;

   /**
    * Storage location of the base of this variable
    *
    * The precise meaning of this field depends on the nature of the variable.
    *
    *   - Vertex shader input: one of the values from \c gl_vert_attrib.
    *   - Vertex shader output: one of the values from \c gl_varying_slot.
    *   - Geometry shader input: one of the values from \c gl_varying_slot.
    *   - Geometry shader output: one of the values from \c gl_varying_slot.
    *   - Fragment shader input: one of the values from \c gl_varying_slot.
    *   - Fragment shader output: one of the values from \c gl_frag_result.
    *   - Uniforms: Per-stage uniform slot number for default uniform block.
    *   - Uniforms: Index within the uniform block definition for UBO members.
    *   - Non-UBO Uniforms: explicit location until linking then reused to
    *     store uniform slot number.
    *   - Other: This field is not currently used.
    *
    * If the variable is a uniform, shader input, or shader output, and the
    * slot has not been assigned, the value will be -1.
    */
   int location;

   /**
    * Specifies the first component the variable is stored in as per
    * ARB_enhanced_layouts.
    */
   unsigned component:2;

   /**
    * Output index for dual source blending.
    *
    * \note
    * The GLSL spec only allows the values 0 or 1 for the index in \b dual
    * source blending.
    */
   unsigned index:1;

   /**
    * Specifies whether a shader input/output is per-patch in tessellation
    * shader stages.
    */
   unsigned patch:1;

   /**
    * Storage class of the variable.
    *
    * \sa (n)ir_variable_mode
    */
   unsigned mode:4;

   /**
    * Interpolation mode for shader inputs / outputs
    *
    * \sa glsl_interp_mode
    */
   unsigned interpolation:2;

   /**
    * Was the location explicitly set in the shader?
    *
    * If the location is explicitly set in the shader, it \b cannot be changed
    * by the linker or by the API (e.g., calls to \c glBindAttribLocation have
    * no effect).
    */
   unsigned explicit_location:1;

   /**
    * Precision qualifier.
    */
   unsigned precision:2;
};

/**
 * Active resource in a gl_shader_program
 */
struct gl_program_resource
{
   GLenum16 Type; /** Program interface type. */
   const void *Data; /** Pointer to resource associated data structure. */
   uint8_t StageReferences; /** Bitmask of shader stage references. */
};

/**
 * Link status enum. LINKING_SKIPPED is used to indicate linking
 * was skipped due to the shader being loaded from the on-disk cache.
 */
enum gl_link_status
{
   LINKING_FAILURE = 0,
   LINKING_SUCCESS,
   LINKING_SKIPPED
};

/**
 * A data structure to be shared by gl_shader_program and gl_program.
 */
struct gl_shader_program_data
{
   GLint RefCount;  /**< Reference count */

   /** SHA1 hash of linked shader program */
   unsigned char sha1[20];

   unsigned NumUniformStorage;
   unsigned NumHiddenUniforms;
   struct gl_uniform_storage *UniformStorage;

   unsigned NumUniformBlocks;
   unsigned NumShaderStorageBlocks;

   struct gl_uniform_block *UniformBlocks;
   struct gl_uniform_block *ShaderStorageBlocks;

   struct gl_active_atomic_buffer *AtomicBuffers;
   unsigned NumAtomicBuffers;

   /* Shader cache variables used during restore */
   unsigned NumUniformDataSlots;
   union gl_constant_value *UniformDataSlots;

   /* Used to hold initial uniform values for program binary restores.
    *
    * From the ARB_get_program_binary spec:
    *
    *    "A successful call to ProgramBinary will reset all uniform
    *    variables to their initial values. The initial value is either
    *    the value of the variable's initializer as specified in the
    *    original shader source, or 0 if no initializer was present.
    */
   union gl_constant_value *UniformDataDefaults;

   /** Hash for quick search by name. */
   struct hash_table_u64 *ProgramResourceHash;

   GLboolean Validated;

   /** List of all active resources after linking. */
   struct gl_program_resource *ProgramResourceList;
   unsigned NumProgramResourceList;

   enum gl_link_status LinkStatus;   /**< GL_LINK_STATUS */
   GLchar *InfoLog;

   unsigned Version;       /**< GLSL version used for linking */

   /* Mask of stages this program was linked against */
   unsigned linked_stages;

   /* Whether the shaders of this program are loaded from SPIR-V binaries
    * (all have the SPIR_V_BINARY_ARB state). This was introduced by the
    * ARB_gl_spirv extension.
    */
   bool spirv;
};

/**
 * A GLSL program object.
 * Basically a linked collection of vertex and fragment shaders.
 */
struct gl_shader_program
{
   GLenum16 Type;   /**< Always GL_SHADER_PROGRAM (internal token) */
   GLuint Name;  /**< aka handle or ID */
   GLchar *Label;   /**< GL_KHR_debug */
   GLint RefCount;  /**< Reference count */
   GLboolean DeletePending;

   /**
    * Is the application intending to glGetProgramBinary this program?
    *
    * BinaryRetrievableHint is the currently active hint that gets set
    * during initialization and after linking and BinaryRetrievableHintPending
    * is the hint set by the user to be active when program is linked next time.
    */
   GLboolean BinaryRetrievableHint;
   GLboolean BinaryRetrievableHintPending;

   /**
    * Indicates whether program can be bound for individual pipeline stages
    * using UseProgramStages after it is next linked.
    */
   GLboolean SeparateShader;

   GLuint NumShaders;          /**< number of attached shaders */
   struct gl_shader **Shaders; /**< List of attached the shaders */

   /**
    * User-defined attribute bindings
    *
    * These are set via \c glBindAttribLocation and are used to direct the
    * GLSL linker.  These are \b not the values used in the compiled shader,
    * and they are \b not the values returned by \c glGetAttribLocation.
    */
   struct string_to_uint_map *AttributeBindings;

   /**
    * User-defined fragment data bindings
    *
    * These are set via \c glBindFragDataLocation and are used to direct the
    * GLSL linker.  These are \b not the values used in the compiled shader,
    * and they are \b not the values returned by \c glGetFragDataLocation.
    */
   struct string_to_uint_map *FragDataBindings;
   struct string_to_uint_map *FragDataIndexBindings;

   /**
    * Transform feedback varyings last specified by
    * glTransformFeedbackVaryings().
    *
    * For the current set of transform feedback varyings used for transform
    * feedback output, see LinkedTransformFeedback.
    */
   struct {
      GLenum16 BufferMode;
      /** Global xfb_stride out qualifier if any */
      GLuint BufferStride[MAX_FEEDBACK_BUFFERS];
      GLuint NumVarying;
      GLchar **VaryingNames;  /**< Array [NumVarying] of char * */
   } TransformFeedback;

   struct gl_program *last_vert_prog;

   /** Post-link gl_FragDepth layout for ARB_conservative_depth. */
   enum gl_frag_depth_layout FragDepthLayout;

   /**
    * Geometry shader state - copied into gl_program by
    * _mesa_copy_linked_program_data().
    */
   struct {
      GLint VerticesIn;

      bool UsesEndPrimitive;
      bool UsesStreams;
   } Geom;

   /**
    * Compute shader state - copied into gl_program by
    * _mesa_copy_linked_program_data().
    */
   struct {
      /**
       * Size of shared variables accessed by the compute shader.
       */
      unsigned SharedSize;
   } Comp;

   /** Data shared by gl_program and gl_shader_program */
   struct gl_shader_program_data *data;

   /**
    * Mapping from GL uniform locations returned by \c glUniformLocation to
    * UniformStorage entries. Arrays will have multiple contiguous slots
    * in the UniformRemapTable, all pointing to the same UniformStorage entry.
    */
   unsigned NumUniformRemapTable;
   struct gl_uniform_storage **UniformRemapTable;

   /**
    * Sometimes there are empty slots left over in UniformRemapTable after we
    * allocate slots to explicit locations. This list stores the blocks of
    * continuous empty slots inside UniformRemapTable.
    */
   struct exec_list EmptyUniformLocations;

   /**
    * Total number of explicit uniform location including inactive uniforms.
    */
   unsigned NumExplicitUniformLocations;

   /**
    * Map of active uniform names to locations
    *
    * Maps any active uniform that is not an array element to a location.
    * Each active uniform, including individual structure members will appear
    * in this map.  This roughly corresponds to the set of names that would be
    * enumerated by \c glGetActiveUniform.
    */
   struct string_to_uint_map *UniformHash;

   GLboolean SamplersValidated; /**< Samplers validated against texture units? */

   bool IsES;              /**< True if this program uses GLSL ES */

   /**
    * Per-stage shaders resulting from the first stage of linking.
    *
    * Set of linked shaders for this program.  The array is accessed using the
    * \c MESA_SHADER_* defines.  Entries for non-existent stages will be
    * \c NULL.
    */
   struct gl_linked_shader *_LinkedShaders[MESA_SHADER_STAGES];

   /**
    * True if any of the fragment shaders attached to this program use:
    * #extension ARB_fragment_coord_conventions: enable
    */
   GLboolean ARB_fragment_coord_conventions_enable;
};


#define GLSL_DUMP      0x1  /**< Dump shaders to stdout */
#define GLSL_LOG       0x2  /**< Write shaders to files */
#define GLSL_UNIFORMS  0x4  /**< Print glUniform calls */
#define GLSL_NOP_VERT  0x8  /**< Force no-op vertex shaders */
#define GLSL_NOP_FRAG 0x10  /**< Force no-op fragment shaders */
#define GLSL_USE_PROG 0x20  /**< Log glUseProgram calls */
#define GLSL_REPORT_ERRORS 0x40  /**< Print compilation errors */
#define GLSL_DUMP_ON_ERROR 0x80 /**< Dump shaders to stderr on compile error */
#define GLSL_CACHE_INFO 0x100 /**< Print debug information about shader cache */
#define GLSL_CACHE_FALLBACK 0x200 /**< Force shader cache fallback paths */


/**
 * Context state for GLSL vertex/fragment shaders.
 * Extended to support pipeline object
 */
struct gl_pipeline_object
{
   /** Name of the pipeline object as received from glGenProgramPipelines.
    * It would be 0 for shaders without separate shader objects.
    */
   GLuint Name;

   GLint RefCount;

   GLchar *Label;   /**< GL_KHR_debug */

   /**
    * Programs used for rendering
    *
    * There is a separate program set for each shader stage.
    */
   struct gl_program *CurrentProgram[MESA_SHADER_STAGES];

   struct gl_shader_program *ReferencedPrograms[MESA_SHADER_STAGES];

   /**
    * Program used by glUniform calls.
    *
    * Explicitly set by \c glUseProgram and \c glActiveProgramEXT.
    */
   struct gl_shader_program *ActiveProgram;

   GLbitfield Flags;         /**< Mask of GLSL_x flags */
   GLboolean EverBound;      /**< Has the pipeline object been created */
   GLboolean Validated;      /**< Pipeline Validation status */

   GLchar *InfoLog;
};

/**
 * Context state for GLSL pipeline shaders.
 */
struct gl_pipeline_shader_state
{
   /** Currently bound pipeline object. See _mesa_BindProgramPipeline() */
   struct gl_pipeline_object *Current;

   /** Default Object to ensure that _Shader is never NULL */
   struct gl_pipeline_object *Default;

   /** Pipeline objects */
   struct _mesa_HashTable *Objects;
};

/**
 * Compiler options for a single GLSL shaders type
 */
struct gl_shader_compiler_options
{
   /** Driver-selectable options: */
   GLboolean EmitNoLoops;
   GLboolean EmitNoCont;                  /**< Emit CONT opcode? */
   GLboolean EmitNoMainReturn;            /**< Emit CONT/RET opcodes? */
   GLboolean EmitNoPow;                   /**< Emit POW opcodes? */
   GLboolean EmitNoSat;                   /**< Emit SAT opcodes? */
   GLboolean LowerCombinedClipCullDistance; /** Lower gl_ClipDistance and
                                              * gl_CullDistance together from
                                              * float[8] to vec4[2]
                                              **/
   GLbitfield LowerBuiltinVariablesXfb;   /**< Which builtin variables should
                                           * be lowered for transform feedback
                                           **/

   /**
    * If we can lower the precision of variables based on precision
    * qualifiers
    */
   GLboolean LowerPrecision;

   /**
    * \name Forms of indirect addressing the driver cannot do.
    */
   /*@{*/
   GLboolean EmitNoIndirectInput;   /**< No indirect addressing of inputs */
   GLboolean EmitNoIndirectOutput;  /**< No indirect addressing of outputs */
   GLboolean EmitNoIndirectTemp;    /**< No indirect addressing of temps */
   GLboolean EmitNoIndirectUniform; /**< No indirect addressing of constants */
   GLboolean EmitNoIndirectSampler; /**< No indirect addressing of samplers */
   /*@}*/

   GLuint MaxIfDepth;               /**< Maximum nested IF blocks */
   GLuint MaxUnrollIterations;

   /**
    * Optimize code for array of structures backends.
    *
    * This is a proxy for:
    *   - preferring DP4 instructions (rather than MUL/MAD) for
    *     matrix * vector operations, such as position transformation.
    */
   GLboolean OptimizeForAOS;

   /** Lower UBO and SSBO access to intrinsics. */
   GLboolean LowerBufferInterfaceBlocks;

   /** Clamp UBO and SSBO block indices so they don't go out-of-bounds. */
   GLboolean ClampBlockIndicesToArrayBounds;

   /** (driconf) Force gl_Position to be considered invariant */
   GLboolean PositionAlwaysInvariant;

   const struct nir_shader_compiler_options *NirOptions;
};


/**
 * Occlusion/timer query object.
 */
struct gl_query_object
{
   GLenum16 Target;    /**< The query target, when active */
   GLuint Id;          /**< hash table ID/name */
   GLchar *Label;      /**< GL_KHR_debug */
   GLuint64EXT Result; /**< the counter */
   GLboolean Active;   /**< inside Begin/EndQuery */
   GLboolean Ready;    /**< result is ready? */
   GLboolean EverBound;/**< has query object ever been bound */
   GLuint Stream;      /**< The stream */
};


/**
 * Context state for query objects.
 */
struct gl_query_state
{
   struct _mesa_HashTable *QueryObjects;
   struct gl_query_object *CurrentOcclusionObject; /* GL_ARB_occlusion_query */
   struct gl_query_object *CurrentTimerObject;     /* GL_EXT_timer_query */

   /** GL_NV_conditional_render */
   struct gl_query_object *CondRenderQuery;

   /** GL_EXT_transform_feedback */
   struct gl_query_object *PrimitivesGenerated[MAX_VERTEX_STREAMS];
   struct gl_query_object *PrimitivesWritten[MAX_VERTEX_STREAMS];

   /** GL_ARB_transform_feedback_overflow_query */
   struct gl_query_object *TransformFeedbackOverflow[MAX_VERTEX_STREAMS];
   struct gl_query_object *TransformFeedbackOverflowAny;

   /** GL_ARB_timer_query */
   struct gl_query_object *TimeElapsed;

   /** GL_ARB_pipeline_statistics_query */
   struct gl_query_object *pipeline_stats[MAX_PIPELINE_STATISTICS];

   GLenum16 CondRenderMode;
};


/** Sync object state */
struct gl_sync_object
{
   GLuint Name;               /**< Fence name */
   GLint RefCount;            /**< Reference count */
   GLchar *Label;             /**< GL_KHR_debug */
   GLboolean DeletePending;   /**< Object was deleted while there were still
                               * live references (e.g., sync not yet finished)
                               */
   GLenum16 SyncCondition;
   GLbitfield Flags;          /**< Flags passed to glFenceSync */
   GLuint StatusFlag:1;       /**< Has the sync object been signaled? */
};


/**
 * State which can be shared by multiple contexts:
 */
struct gl_shared_state
{
   simple_mtx_t Mutex;		   /**< for thread safety */
   GLint RefCount;			   /**< Reference count */
   struct _mesa_HashTable *DisplayList;	   /**< Display lists hash table */
   struct _mesa_HashTable *BitmapAtlas;    /**< For optimized glBitmap text */
   struct _mesa_HashTable *TexObjects;	   /**< Texture objects hash table */

   /** Default texture objects (shared by all texture units) */
   struct gl_texture_object *DefaultTex[NUM_TEXTURE_TARGETS];

   /** Fallback texture used when a bound texture is incomplete */
   struct gl_texture_object *FallbackTex[NUM_TEXTURE_TARGETS];

   /**
    * \name Thread safety and statechange notification for texture
    * objects.
    *
    * \todo Improve the granularity of locking.
    */
   /*@{*/
   mtx_t TexMutex;		/**< texobj thread safety */
   GLuint TextureStateStamp;	        /**< state notification for shared tex */
   /*@}*/

   /**
    * \name Vertex/geometry/fragment programs
    */
   /*@{*/
   struct _mesa_HashTable *Programs; /**< All vertex/fragment programs */
   struct gl_program *DefaultVertexProgram;
   struct gl_program *DefaultFragmentProgram;
   /*@}*/

   /* GL_ATI_fragment_shader */
   struct _mesa_HashTable *ATIShaders;
   struct ati_fragment_shader *DefaultFragmentShader;

   struct _mesa_HashTable *BufferObjects;

   /** Table of both gl_shader and gl_shader_program objects */
   struct _mesa_HashTable *ShaderObjects;

   /* GL_EXT_framebuffer_object */
   struct _mesa_HashTable *RenderBuffers;
   struct _mesa_HashTable *FrameBuffers;

   /* GL_ARB_sync */
   struct set *SyncObjects;

   /** GL_ARB_sampler_objects */
   struct _mesa_HashTable *SamplerObjects;

   /* GL_ARB_bindless_texture */
   struct hash_table_u64 *TextureHandles;
   struct hash_table_u64 *ImageHandles;
   mtx_t HandlesMutex; /**< For texture/image handles safety */

   /* GL_ARB_shading_language_include */
   struct shader_includes *ShaderIncludes;
   /* glCompileShaderInclude expects ShaderIncludes not to change while it is
    * in progress.
    */
   mtx_t ShaderIncludeMutex;

   /**
    * Some context in this share group was affected by a GPU reset
    *
    * On the next call to \c glGetGraphicsResetStatus, contexts that have not
    * been affected by a GPU reset must also return
    * \c GL_INNOCENT_CONTEXT_RESET_ARB.
    *
    * Once this field becomes true, it is never reset to false.
    */
   bool ShareGroupReset;

   /** EXT_external_objects */
   struct _mesa_HashTable *MemoryObjects;

   /** EXT_semaphore */
   struct _mesa_HashTable *SemaphoreObjects;

   /**
    * Some context in this share group was affected by a disjoint
    * operation. This operation can be anything that has effects on
    * values of timer queries in such manner that they become invalid for
    * performance metrics. As example gpu reset, counter overflow or gpu
    * frequency changes.
    */
   bool DisjointOperation;
};



/**
 * Renderbuffers represent drawing surfaces such as color, depth and/or
 * stencil.  A framebuffer object has a set of renderbuffers.
 * Drivers will typically derive subclasses of this type.
 */
struct gl_renderbuffer
{
   simple_mtx_t Mutex; /**< for thread safety */
   GLuint ClassID;        /**< Useful for drivers */
   GLuint Name;
   GLchar *Label;         /**< GL_KHR_debug */
   GLint RefCount;
   GLuint Width, Height;
   GLuint Depth;
   GLboolean Purgeable;  /**< Is the buffer purgeable under memory pressure? */
   GLboolean AttachedAnytime; /**< TRUE if it was attached to a framebuffer */
   /**
    * True for renderbuffers that wrap textures, giving the driver a chance to
    * flush render caches through the FinishRenderTexture hook.
    *
    * Drivers may also set this on renderbuffers other than those generated by
    * glFramebufferTexture(), though it means FinishRenderTexture() would be
    * called without a rb->TexImage.
    */
   GLboolean NeedsFinishRenderTexture;
   GLubyte NumSamples;    /**< zero means not multisampled */
   GLubyte NumStorageSamples; /**< for AMD_framebuffer_multisample_advanced */
   GLenum16 InternalFormat; /**< The user-specified format */
   GLenum16 _BaseFormat;    /**< Either GL_RGB, GL_RGBA, GL_DEPTH_COMPONENT or
                               GL_STENCIL_INDEX. */
   mesa_format Format;      /**< The actual renderbuffer memory format */
   /**
    * Pointer to the texture image if this renderbuffer wraps a texture,
    * otherwise NULL.
    *
    * Note that the reference on the gl_texture_object containing this
    * TexImage is held by the gl_renderbuffer_attachment.
    */
   struct gl_texture_image *TexImage;

   /** Delete this renderbuffer */
   void (*Delete)(struct gl_context *ctx, struct gl_renderbuffer *rb);

   /** Allocate new storage for this renderbuffer */
   GLboolean (*AllocStorage)(struct gl_context *ctx,
                             struct gl_renderbuffer *rb,
                             GLenum internalFormat,
                             GLuint width, GLuint height);
};


/**
 * A renderbuffer attachment points to either a texture object (and specifies
 * a mipmap level, cube face or 3D texture slice) or points to a renderbuffer.
 */
struct gl_renderbuffer_attachment
{
   GLenum16 Type; /**< \c GL_NONE or \c GL_TEXTURE or \c GL_RENDERBUFFER_EXT */
   GLboolean Complete;

   /**
    * If \c Type is \c GL_RENDERBUFFER_EXT, this stores a pointer to the
    * application supplied renderbuffer object.
    */
   struct gl_renderbuffer *Renderbuffer;

   /**
    * If \c Type is \c GL_TEXTURE, this stores a pointer to the application
    * supplied texture object.
    */
   struct gl_texture_object *Texture;
   GLuint TextureLevel; /**< Attached mipmap level. */
   GLsizei NumSamples;  /**< from FramebufferTexture2DMultisampleEXT */
   GLuint CubeMapFace;  /**< 0 .. 5, for cube map textures. */
   GLuint Zoffset;      /**< Slice for 3D textures,  or layer for both 1D
                         * and 2D array textures */
   GLboolean Layered;
};


/**
 * A framebuffer is a collection of renderbuffers (color, depth, stencil, etc).
 * In C++ terms, think of this as a base class from which device drivers
 * will make derived classes.
 */
struct gl_framebuffer
{
   simple_mtx_t Mutex;  /**< for thread safety */
   /**
    * If zero, this is a window system framebuffer.  If non-zero, this
    * is a FBO framebuffer; note that for some devices (i.e. those with
    * a natural pixel coordinate system for FBOs that differs from the
    * OpenGL/Mesa coordinate system), this means that the viewport,
    * polygon face orientation, and polygon stipple will have to be inverted.
    */
   GLuint Name;
   GLint RefCount;

   GLchar *Label;       /**< GL_KHR_debug */

   GLboolean DeletePending;

   /**
    * The framebuffer's visual. Immutable if this is a window system buffer.
    * Computed from attachments if user-made FBO.
    */
   struct gl_config Visual;

   /**
    * Size of frame buffer in pixels. If there are no attachments, then both
    * of these are 0.
    */
   GLuint Width, Height;

   /**
    * In the case that the framebuffer has no attachment (i.e.
    * GL_ARB_framebuffer_no_attachments) then the geometry of
    * the framebuffer is specified by the default values.
    */
   struct {
     GLuint Width, Height, Layers, NumSamples;
     GLboolean FixedSampleLocations;
     /* Derived from NumSamples by the driver so that it can choose a valid
      * value for the hardware.
      */
     GLuint _NumSamples;
   } DefaultGeometry;

   /** \name  Drawing bounds (Intersection of buffer size and scissor box)
    * The drawing region is given by [_Xmin, _Xmax) x [_Ymin, _Ymax),
    * (inclusive for _Xmin and _Ymin while exclusive for _Xmax and _Ymax)
    */
   /*@{*/
   GLint _Xmin, _Xmax;
   GLint _Ymin, _Ymax;
   /*@}*/

   /** \name  Derived Z buffer stuff */
   /*@{*/
   GLuint _DepthMax;	/**< Max depth buffer value */
   GLfloat _DepthMaxF;	/**< Float max depth buffer value */
   GLfloat _MRD;	/**< minimum resolvable difference in Z values */
   /*@}*/

   /** One of the GL_FRAMEBUFFER_(IN)COMPLETE_* tokens */
   GLenum16 _Status;

   /** Whether one of Attachment has Type != GL_NONE
    * NOTE: the values for Width and Height are set to 0 in case of having
    * no attachments, a backend driver supporting the extension
    * GL_ARB_framebuffer_no_attachments must check for the flag _HasAttachments
    * and if GL_FALSE, must then use the values in DefaultGeometry to initialize
    * its viewport, scissor and so on (in particular _Xmin, _Xmax, _Ymin and
    * _Ymax do NOT take into account _HasAttachments being false). To get the
    * geometry of the framebuffer, the  helper functions
    *   _mesa_geometric_width(),
    *   _mesa_geometric_height(),
    *   _mesa_geometric_samples() and
    *   _mesa_geometric_layers()
    * are available that check _HasAttachments.
    */
   bool _HasAttachments;

   GLbitfield _IntegerBuffers;  /**< Which color buffers are integer valued */
   GLbitfield _RGBBuffers;  /**< Which color buffers have baseformat == RGB */
   GLbitfield _FP32Buffers; /**< Which color buffers are FP32 */

   /* ARB_color_buffer_float */
   GLboolean _AllColorBuffersFixedPoint; /* no integer, no float */
   GLboolean _HasSNormOrFloatColorBuffer;

   /**
    * The maximum number of layers in the framebuffer, or 0 if the framebuffer
    * is not layered.  For cube maps and cube map arrays, each cube face
    * counts as a layer. As the case for Width, Height a backend driver
    * supporting GL_ARB_framebuffer_no_attachments must use DefaultGeometry
    * in the case that _HasAttachments is false
    */
   GLuint MaxNumLayers;

   /** Array of all renderbuffer attachments, indexed by BUFFER_* tokens. */
   struct gl_renderbuffer_attachment Attachment[BUFFER_COUNT];

   /* In unextended OpenGL these vars are part of the GL_COLOR_BUFFER
    * attribute group and GL_PIXEL attribute group, respectively.
    */
   GLenum16 ColorDrawBuffer[MAX_DRAW_BUFFERS];
   GLenum16 ColorReadBuffer;

   /* GL_ARB_sample_locations */
   GLfloat *SampleLocationTable; /**< If NULL, no table has been specified */
   GLboolean ProgrammableSampleLocations;
   GLboolean SampleLocationPixelGrid;

   /** Computed from ColorDraw/ReadBuffer above */
   GLuint _NumColorDrawBuffers;
   gl_buffer_index _ColorDrawBufferIndexes[MAX_DRAW_BUFFERS];
   gl_buffer_index _ColorReadBufferIndex;
   struct gl_renderbuffer *_ColorDrawBuffers[MAX_DRAW_BUFFERS];
   struct gl_renderbuffer *_ColorReadBuffer;

   /* GL_MESA_framebuffer_flip_y */
   bool FlipY;

   /** Delete this framebuffer */
   void (*Delete)(struct gl_framebuffer *fb);
};


/**
 * Precision info for shader datatypes.  See glGetShaderPrecisionFormat().
 */
struct gl_precision
{
   GLushort RangeMin;   /**< min value exponent */
   GLushort RangeMax;   /**< max value exponent */
   GLushort Precision;  /**< number of mantissa bits */
};


/**
 * Limits for vertex, geometry and fragment programs/shaders.
 */
struct gl_program_constants
{
   /* logical limits */
   GLuint MaxInstructions;
   GLuint MaxAluInstructions;
   GLuint MaxTexInstructions;
   GLuint MaxTexIndirections;
   GLuint MaxAttribs;
   GLuint MaxTemps;
   GLuint MaxAddressRegs;
   GLuint MaxAddressOffset;  /**< [-MaxAddressOffset, MaxAddressOffset-1] */
   GLuint MaxParameters;
   GLuint MaxLocalParams;
   GLuint MaxEnvParams;
   /* native/hardware limits */
   GLuint MaxNativeInstructions;
   GLuint MaxNativeAluInstructions;
   GLuint MaxNativeTexInstructions;
   GLuint MaxNativeTexIndirections;
   GLuint MaxNativeAttribs;
   GLuint MaxNativeTemps;
   GLuint MaxNativeAddressRegs;
   GLuint MaxNativeParameters;
   /* For shaders */
   GLuint MaxUniformComponents;  /**< Usually == MaxParameters * 4 */

   /**
    * \name Per-stage input / output limits
    *
    * Previous to OpenGL 3.2, the intrastage data limits were advertised with
    * a single value: GL_MAX_VARYING_COMPONENTS (GL_MAX_VARYING_VECTORS in
    * ES).  This is stored as \c gl_constants::MaxVarying.
    *
    * Starting with OpenGL 3.2, the limits are advertised with per-stage
    * variables.  Each stage as a certain number of outputs that it can feed
    * to the next stage and a certain number inputs that it can consume from
    * the previous stage.
    *
    * Vertex shader inputs do not participate this in this accounting.
    * These are tracked exclusively by \c gl_program_constants::MaxAttribs.
    *
    * Fragment shader outputs do not participate this in this accounting.
    * These are tracked exclusively by \c gl_constants::MaxDrawBuffers.
    */
   /*@{*/
   GLuint MaxInputComponents;
   GLuint MaxOutputComponents;
   /*@}*/

   /* ES 2.0 and GL_ARB_ES2_compatibility */
   struct gl_precision LowFloat, MediumFloat, HighFloat;
   struct gl_precision LowInt, MediumInt, HighInt;
   /* GL_ARB_uniform_buffer_object */
   GLuint MaxUniformBlocks;
   uint64_t MaxCombinedUniformComponents;
   GLuint MaxTextureImageUnits;

   /* GL_ARB_shader_atomic_counters */
   GLuint MaxAtomicBuffers;
   GLuint MaxAtomicCounters;

   /* GL_ARB_shader_image_load_store */
   GLuint MaxImageUniforms;

   /* GL_ARB_shader_storage_buffer_object */
   GLuint MaxShaderStorageBlocks;
};

/**
 * Constants which may be overridden by device driver during context creation
 * but are never changed after that.
 */
struct gl_constants
{
   GLuint MaxTextureMbytes;      /**< Max memory per image, in MB */
   GLuint MaxTextureSize;        /**< Max 1D/2D texture size, in pixels*/
   GLuint Max3DTextureLevels;    /**< Max mipmap levels for 3D textures */
   GLuint MaxCubeTextureLevels;  /**< Max mipmap levels for cube textures */
   GLuint MaxArrayTextureLayers; /**< Max layers in array textures */
   GLuint MaxTextureRectSize;    /**< Max rectangle texture size, in pixes */
   GLuint MaxTextureCoordUnits;
   GLuint MaxCombinedTextureImageUnits;
   GLuint MaxTextureUnits; /**< = MIN(CoordUnits, FragmentProgram.ImageUnits) */
   GLfloat MaxTextureMaxAnisotropy;  /**< GL_EXT_texture_filter_anisotropic */
   GLfloat MaxTextureLodBias;        /**< GL_EXT_texture_lod_bias */
   GLuint MaxTextureBufferSize;      /**< GL_ARB_texture_buffer_object */

   GLuint TextureBufferOffsetAlignment; /**< GL_ARB_texture_buffer_range */

   GLuint MaxArrayLockSize;

   GLint SubPixelBits;

   GLfloat MinPointSize, MaxPointSize;	     /**< aliased */
   GLfloat MinPointSizeAA, MaxPointSizeAA;   /**< antialiased */
   GLfloat PointSizeGranularity;
   GLfloat MinLineWidth, MaxLineWidth;       /**< aliased */
   GLfloat MinLineWidthAA, MaxLineWidthAA;   /**< antialiased */
   GLfloat LineWidthGranularity;

   GLuint MaxClipPlanes;
   GLuint MaxLights;
   GLfloat MaxShininess;                     /**< GL_NV_light_max_exponent */
   GLfloat MaxSpotExponent;                  /**< GL_NV_light_max_exponent */

   GLuint MaxViewportWidth, MaxViewportHeight;
   GLuint MaxViewports;                      /**< GL_ARB_viewport_array */
   GLuint ViewportSubpixelBits;              /**< GL_ARB_viewport_array */
   struct {
      GLfloat Min;
      GLfloat Max;
   } ViewportBounds;                         /**< GL_ARB_viewport_array */
   GLuint MaxWindowRectangles;               /**< GL_EXT_window_rectangles */

   struct gl_program_constants Program[MESA_SHADER_STAGES];
   GLuint MaxProgramMatrices;
   GLuint MaxProgramMatrixStackDepth;

   struct {
      GLuint SamplesPassed;
      GLuint TimeElapsed;
      GLuint Timestamp;
      GLuint PrimitivesGenerated;
      GLuint PrimitivesWritten;
      GLuint VerticesSubmitted;
      GLuint PrimitivesSubmitted;
      GLuint VsInvocations;
      GLuint TessPatches;
      GLuint TessInvocations;
      GLuint GsInvocations;
      GLuint GsPrimitives;
      GLuint FsInvocations;
      GLuint ComputeInvocations;
      GLuint ClInPrimitives;
      GLuint ClOutPrimitives;
   } QueryCounterBits;

   GLuint MaxDrawBuffers;    /**< GL_ARB_draw_buffers */

   GLuint MaxColorAttachments;   /**< GL_EXT_framebuffer_object */
   GLuint MaxRenderbufferSize;   /**< GL_EXT_framebuffer_object */
   GLuint MaxSamples;            /**< GL_ARB_framebuffer_object */

   /**
    * GL_ARB_framebuffer_no_attachments
    */
   GLuint MaxFramebufferWidth;
   GLuint MaxFramebufferHeight;
   GLuint MaxFramebufferLayers;
   GLuint MaxFramebufferSamples;

   /** Number of varying vectors between any two shader stages. */
   GLuint MaxVarying;

   /** @{
    * GL_ARB_uniform_buffer_object
    */
   GLuint MaxCombinedUniformBlocks;
   GLuint MaxUniformBufferBindings;
   GLuint MaxUniformBlockSize;
   GLuint UniformBufferOffsetAlignment;
   /** @} */

   /** @{
    * GL_ARB_shader_storage_buffer_object
    */
   GLuint MaxCombinedShaderStorageBlocks;
   GLuint MaxShaderStorageBufferBindings;
   GLuint MaxShaderStorageBlockSize;
   GLuint ShaderStorageBufferOffsetAlignment;
   /** @} */

   /**
    * GL_ARB_explicit_uniform_location
    */
   GLuint MaxUserAssignableUniformLocations;

   /** geometry shader */
   GLuint MaxGeometryOutputVertices;
   GLuint MaxGeometryTotalOutputComponents;
   GLuint MaxGeometryShaderInvocations;

   GLuint GLSLVersion;  /**< Desktop GLSL version supported (ex: 120 = 1.20) */
   GLuint GLSLVersionCompat;  /**< Desktop compat GLSL version supported  */

   /**
    * Changes default GLSL extension behavior from "error" to "warn".  It's out
    * of spec, but it can make some apps work that otherwise wouldn't.
    */
   GLboolean ForceGLSLExtensionsWarn;

   /**
    * If non-zero, forces GLSL shaders to behave as if they began
    * with "#version ForceGLSLVersion".
    */
   GLuint ForceGLSLVersion;

   /**
    * Allow GLSL #extension directives in the middle of shaders.
    */
   GLboolean AllowGLSLExtensionDirectiveMidShader;

   /**
    * Allow builtins as part of constant expressions. This was not allowed
    * until GLSL 1.20 this allows it everywhere.
    */
   GLboolean AllowGLSLBuiltinConstantExpression;

   /**
    * Allow some relaxation of GLSL ES shader restrictions. This encompasses
    * a number of relaxations to the ES shader rules.
    */
   GLboolean AllowGLSLRelaxedES;

   /**
    * Allow GLSL built-in variables to be redeclared verbatim
    */
   GLboolean AllowGLSLBuiltinVariableRedeclaration;

   /**
    * Allow GLSL interpolation qualifier mismatch across shader stages.
    */
   GLboolean AllowGLSLCrossStageInterpolationMismatch;

   /**
    * Allow creating a higher compat profile (version 3.1+) for apps that
    * request it. Be careful when adding that driconf option because some
    * features are unimplemented and might not work correctly.
    */
   GLboolean AllowHigherCompatVersion;

   /**
    * Allow layout qualifiers on function parameters.
    */
   GLboolean AllowLayoutQualifiersOnFunctionParameters;

   /**
    * Force computing the absolute value for sqrt() and inversesqrt() to follow
    * D3D9 when apps rely on this behaviour.
    */
   GLboolean ForceGLSLAbsSqrt;

   /**
    * Force uninitialized variables to default to zero.
    */
   GLboolean GLSLZeroInit;

   /**
    * Treat integer textures using GL_LINEAR filters as GL_NEAREST.
    */
   GLboolean ForceIntegerTexNearest;

   /**
    * Does the driver support real 32-bit integers?  (Otherwise, integers are
    * simulated via floats.)
    */
   GLboolean NativeIntegers;

   /**
    * Does VertexID count from zero or from base vertex?
    *
    * \note
    * If desktop GLSL 1.30 or GLSL ES 3.00 are not supported, this field is
    * ignored and need not be set.
    */
   bool VertexID_is_zero_based;

   /**
    * If the driver supports real 32-bit integers, what integer value should be
    * used for boolean true in uniform uploads?  (Usually 1 or ~0.)
    */
   GLuint UniformBooleanTrue;

   /**
    * Maximum amount of time, measured in nanseconds, that the server can wait.
    */
   GLuint64 MaxServerWaitTimeout;

   /** GL_EXT_provoking_vertex */
   GLboolean QuadsFollowProvokingVertexConvention;

   /** GL_ARB_viewport_array */
   GLenum16 LayerAndVPIndexProvokingVertex;

   /** OpenGL version 3.0 */
   GLbitfield ContextFlags;  /**< Ex: GL_CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT */

   /** OpenGL version 3.2 */
   GLbitfield ProfileMask;   /**< Mask of CONTEXT_x_PROFILE_BIT */

   /** OpenGL version 4.4 */
   GLuint MaxVertexAttribStride;

   /** GL_EXT_transform_feedback */
   GLuint MaxTransformFeedbackBuffers;
   GLuint MaxTransformFeedbackSeparateComponents;
   GLuint MaxTransformFeedbackInterleavedComponents;
   GLuint MaxVertexStreams;

   /** GL_EXT_gpu_shader4 */
   GLint MinProgramTexelOffset, MaxProgramTexelOffset;

   /** GL_ARB_texture_gather */
   GLuint MinProgramTextureGatherOffset;
   GLuint MaxProgramTextureGatherOffset;
   GLuint MaxProgramTextureGatherComponents;

   /* GL_ARB_robustness */
   GLenum16 ResetStrategy;

   /* GL_KHR_robustness */
   GLboolean RobustAccess;

   /* GL_ARB_blend_func_extended */
   GLuint MaxDualSourceDrawBuffers;

   /**
    * Whether the implementation strips out and ignores texture borders.
    *
    * Many GPU hardware implementations don't support rendering with texture
    * borders and mipmapped textures.  (Note: not static border color, but the
    * old 1-pixel border around each edge).  Implementations then have to do
    * slow fallbacks to be correct, or just ignore the border and be fast but
    * wrong.  Setting the flag strips the border off of TexImage calls,
    * providing "fast but wrong" at significantly reduced driver complexity.
    *
    * Texture borders are deprecated in GL 3.0.
    **/
   GLboolean StripTextureBorder;

   /**
    * For drivers which can do a better job at eliminating unused uniforms
    * than the GLSL compiler.
    *
    * XXX Remove these as soon as a better solution is available.
    */
   GLboolean GLSLSkipStrictMaxUniformLimitCheck;

   /**
    * Whether gl_FragCoord, gl_PointCoord and gl_FrontFacing
    * are system values.
    **/
   bool GLSLFragCoordIsSysVal;
   bool GLSLPointCoordIsSysVal;
   bool GLSLFrontFacingIsSysVal;

   /**
    * Run the minimum amount of GLSL optimizations to be able to link
    * shaders optimally (eliminate dead varyings and uniforms) and just do
    * all the necessary lowering.
    */
   bool GLSLOptimizeConservatively;

   /**
    * Whether to call lower_const_arrays_to_uniforms() during linking.
    */
   bool GLSLLowerConstArrays;

   /**
    * True if gl_TessLevelInner/Outer[] in the TES should be inputs
    * (otherwise, they're system values).
    */
   bool GLSLTessLevelsAsInputs;

   /**
    * Always use the GetTransformFeedbackVertexCount() driver hook, rather
    * than passing the transform feedback object to the drawing function.
    */
   GLboolean AlwaysUseGetTransformFeedbackVertexCount;

   /** GL_ARB_map_buffer_alignment */
   GLuint MinMapBufferAlignment;

   /**
    * Disable varying packing.  This is out of spec, but potentially useful
    * for older platforms that supports a limited number of texture
    * indirections--on these platforms, unpacking the varyings in the fragment
    * shader increases the number of texture indirections by 1, which might
    * make some shaders not executable at all.
    *
    * Drivers that support transform feedback must set this value to GL_FALSE.
    */
   GLboolean DisableVaryingPacking;

   /**
    * Disable varying packing if used for transform feedback.  This is needed
    * for some drivers (e.g. Panfrost) where transform feedback requires
    * unpacked varyings.
    *
    * This variable is mutually exlusive with DisableVaryingPacking.
    */
   GLboolean DisableTransformFeedbackPacking;

   /**
    * UBOs and SSBOs can be packed tightly by the OpenGL implementation when
    * layout is set as shared (the default) or packed. However most Mesa drivers
    * just use STD140 for these layouts. This flag allows drivers to use STD430
    * for packed and shared layouts which allows arrays to be packed more
    * tightly.
    */
   bool UseSTD430AsDefaultPacking;

   /**
    * Should meaningful names be generated for compiler temporary variables?
    *
    * Generally, it is not useful to have the compiler generate "meaningful"
    * names for temporary variables that it creates.  This can, however, be a
    * useful debugging aid.  In Mesa debug builds or release builds when
    * MESA_GLSL is set at run-time, meaningful names will be generated.
    * Drivers can also force names to be generated by setting this field.
    * For example, the i965 driver may set it when INTEL_DEBUG=vs (to dump
    * vertex shader assembly) is set at run-time.
    */
   bool GenerateTemporaryNames;

   /*
    * Maximum value supported for an index in DrawElements and friends.
    *
    * This must be at least (1ull<<24)-1.  The default value is
    * (1ull<<32)-1.
    *
    * \since ES 3.0 or GL_ARB_ES3_compatibility
    * \sa _mesa_init_constants
    */
   GLuint64 MaxElementIndex;

   /**
    * Disable interpretation of line continuations (lines ending with a
    * backslash character ('\') in GLSL source.
    */
   GLboolean DisableGLSLLineContinuations;

   /** GL_ARB_texture_multisample */
   GLint MaxColorTextureSamples;
   GLint MaxDepthTextureSamples;
   GLint MaxIntegerSamples;

   /** GL_AMD_framebuffer_multisample_advanced */
   GLint MaxColorFramebufferSamples;
   GLint MaxColorFramebufferStorageSamples;
   GLint MaxDepthStencilFramebufferSamples;

   /* An array of supported MSAA modes allowing different sample
    * counts per attachment type.
    */
   struct {
      GLint NumColorSamples;
      GLint NumColorStorageSamples;
      GLint NumDepthStencilSamples;
   } SupportedMultisampleModes[40];
   GLint NumSupportedMultisampleModes;

   /** GL_ARB_shader_atomic_counters */
   GLuint MaxAtomicBufferBindings;
   GLuint MaxAtomicBufferSize;
   GLuint MaxCombinedAtomicBuffers;
   GLuint MaxCombinedAtomicCounters;

   /** GL_ARB_vertex_attrib_binding */
   GLint MaxVertexAttribRelativeOffset;
   GLint MaxVertexAttribBindings;

   /* GL_ARB_shader_image_load_store */
   GLuint MaxImageUnits;
   GLuint MaxCombinedShaderOutputResources;
   GLuint MaxImageSamples;
   GLuint MaxCombinedImageUniforms;

   /** GL_ARB_compute_shader */
   GLuint MaxComputeWorkGroupCount[3]; /* Array of x, y, z dimensions */
   GLuint MaxComputeWorkGroupSize[3]; /* Array of x, y, z dimensions */
   GLuint MaxComputeWorkGroupInvocations;
   GLuint MaxComputeSharedMemorySize;

   /** GL_ARB_compute_variable_group_size */
   GLuint MaxComputeVariableGroupSize[3]; /* Array of x, y, z dimensions */
   GLuint MaxComputeVariableGroupInvocations;

   /** GL_ARB_gpu_shader5 */
   GLfloat MinFragmentInterpolationOffset;
   GLfloat MaxFragmentInterpolationOffset;

   GLboolean FakeSWMSAA;

   /** GL_KHR_context_flush_control */
   GLenum16 ContextReleaseBehavior;

   struct gl_shader_compiler_options ShaderCompilerOptions[MESA_SHADER_STAGES];

   /** GL_ARB_tessellation_shader */
   GLuint MaxPatchVertices;
   GLuint MaxTessGenLevel;
   GLuint MaxTessPatchComponents;
   GLuint MaxTessControlTotalOutputComponents;
   bool LowerTessLevel; /**< Lower gl_TessLevel* from float[n] to vecn? */
   bool PrimitiveRestartForPatches;
   bool LowerCsDerivedVariables;    /**< Lower gl_GlobalInvocationID and
                                     *   gl_LocalInvocationIndex based on
                                     *   other builtin variables. */

   /** GL_OES_primitive_bounding_box */
   bool NoPrimitiveBoundingBoxOutput;

   /** GL_ARB_sparse_buffer */
   GLuint SparseBufferPageSize;

   /** Used as an input for sha1 generation in the on-disk shader cache */
   unsigned char *dri_config_options_sha1;

   /** When drivers are OK with mapped buffers during draw and other calls. */
   bool AllowMappedBuffersDuringExecution;

   /**
    * Whether buffer creation, unsynchronized mapping, unmapping, and
    * deletion is thread-safe.
    */
   bool BufferCreateMapUnsynchronizedThreadSafe;

   /** GL_ARB_get_program_binary */
   GLuint NumProgramBinaryFormats;

   /** GL_NV_conservative_raster */
   GLuint MaxSubpixelPrecisionBiasBits;

   /** GL_NV_conservative_raster_dilate */
   GLfloat ConservativeRasterDilateRange[2];
   GLfloat ConservativeRasterDilateGranularity;

   /** Is the drivers uniform storage packed or padded to 16 bytes. */
   bool PackedDriverUniformStorage;

   /** Does the driver make use of the NIR based GLSL linker */
   bool UseNIRGLSLLinker;

   /** Wether or not glBitmap uses red textures rather than alpha */
   bool BitmapUsesRed;

   /** Whether the vertex buffer offset is a signed 32-bit integer. */
   bool VertexBufferOffsetIsInt32;

   /** Whether the driver can handle MultiDrawElements with non-VBO indices. */
   bool MultiDrawWithUserIndices;

   /** Whether out-of-order draw (Begin/End) optimizations are allowed. */
   bool AllowDrawOutOfOrder;

   /** GL_ARB_gl_spirv */
   struct spirv_supported_capabilities SpirVCapabilities;

   /** GL_ARB_spirv_extensions */
   struct spirv_supported_extensions *SpirVExtensions;

   char *VendorOverride;

   /** Buffer size used to upload vertices from glBegin/glEnd. */
   unsigned glBeginEndBufferSize;
};


/**
 * Enable flag for each OpenGL extension.  Different device drivers will
 * enable different extensions at runtime.
 */
struct gl_extensions
{
   GLboolean dummy;  /* don't remove this! */
   GLboolean dummy_true;  /* Set true by _mesa_init_extensions(). */
   GLboolean dummy_false; /* Set false by _mesa_init_extensions(). */
   GLboolean ANGLE_texture_compression_dxt;
   GLboolean ARB_ES2_compatibility;
   GLboolean ARB_ES3_compatibility;
   GLboolean ARB_ES3_1_compatibility;
   GLboolean ARB_ES3_2_compatibility;
   GLboolean ARB_arrays_of_arrays;
   GLboolean ARB_base_instance;
   GLboolean ARB_bindless_texture;
   GLboolean ARB_blend_func_extended;
   GLboolean ARB_buffer_storage;
   GLboolean ARB_clear_texture;
   GLboolean ARB_clip_control;
   GLboolean ARB_color_buffer_float;
   GLboolean ARB_compatibility;
   GLboolean ARB_compute_shader;
   GLboolean ARB_compute_variable_group_size;
   GLboolean ARB_conditional_render_inverted;
   GLboolean ARB_conservative_depth;
   GLboolean ARB_copy_image;
   GLboolean ARB_cull_distance;
   GLboolean ARB_depth_buffer_float;
   GLboolean ARB_depth_clamp;
   GLboolean ARB_depth_texture;
   GLboolean ARB_derivative_control;
   GLboolean ARB_draw_buffers_blend;
   GLboolean ARB_draw_elements_base_vertex;
   GLboolean ARB_draw_indirect;
   GLboolean ARB_draw_instanced;
   GLboolean ARB_fragment_coord_conventions;
   GLboolean ARB_fragment_layer_viewport;
   GLboolean ARB_fragment_program;
   GLboolean ARB_fragment_program_shadow;
   GLboolean ARB_fragment_shader;
   GLboolean ARB_framebuffer_no_attachments;
   GLboolean ARB_framebuffer_object;
   GLboolean ARB_fragment_shader_interlock;
   GLboolean ARB_enhanced_layouts;
   GLboolean ARB_explicit_attrib_location;
   GLboolean ARB_explicit_uniform_location;
   GLboolean ARB_gl_spirv;
   GLboolean ARB_gpu_shader5;
   GLboolean ARB_gpu_shader_fp64;
   GLboolean ARB_gpu_shader_int64;
   GLboolean ARB_half_float_vertex;
   GLboolean ARB_indirect_parameters;
   GLboolean ARB_instanced_arrays;
   GLboolean ARB_internalformat_query;
   GLboolean ARB_internalformat_query2;
   GLboolean ARB_map_buffer_range;
   GLboolean ARB_occlusion_query;
   GLboolean ARB_occlusion_query2;
   GLboolean ARB_pipeline_statistics_query;
   GLboolean ARB_point_sprite;
   GLboolean ARB_polygon_offset_clamp;
   GLboolean ARB_post_depth_coverage;
   GLboolean ARB_query_buffer_object;
   GLboolean ARB_robust_buffer_access_behavior;
   GLboolean ARB_sample_locations;
   GLboolean ARB_sample_shading;
   GLboolean ARB_seamless_cube_map;
   GLboolean ARB_shader_atomic_counter_ops;
   GLboolean ARB_shader_atomic_counters;
   GLboolean ARB_shader_ballot;
   GLboolean ARB_shader_bit_encoding;
   GLboolean ARB_shader_clock;
   GLboolean ARB_shader_draw_parameters;
   GLboolean ARB_shader_group_vote;
   GLboolean ARB_shader_image_load_store;
   GLboolean ARB_shader_image_size;
   GLboolean ARB_shader_precision;
   GLboolean ARB_shader_stencil_export;
   GLboolean ARB_shader_storage_buffer_object;
   GLboolean ARB_shader_texture_image_samples;
   GLboolean ARB_shader_texture_lod;
   GLboolean ARB_shader_viewport_layer_array;
   GLboolean ARB_shading_language_packing;
   GLboolean ARB_shading_language_420pack;
   GLboolean ARB_shadow;
   GLboolean ARB_sparse_buffer;
   GLboolean ARB_stencil_texturing;
   GLboolean ARB_spirv_extensions;
   GLboolean ARB_sync;
   GLboolean ARB_tessellation_shader;
   GLboolean ARB_texture_border_clamp;
   GLboolean ARB_texture_buffer_object;
   GLboolean ARB_texture_buffer_object_rgb32;
   GLboolean ARB_texture_buffer_range;
   GLboolean ARB_texture_compression_bptc;
   GLboolean ARB_texture_compression_rgtc;
   GLboolean ARB_texture_cube_map;
   GLboolean ARB_texture_cube_map_array;
   GLboolean ARB_texture_env_combine;
   GLboolean ARB_texture_env_crossbar;
   GLboolean ARB_texture_env_dot3;
   GLboolean ARB_texture_filter_anisotropic;
   GLboolean ARB_texture_float;
   GLboolean ARB_texture_gather;
   GLboolean ARB_texture_mirror_clamp_to_edge;
   GLboolean ARB_texture_multisample;
   GLboolean ARB_texture_non_power_of_two;
   GLboolean ARB_texture_stencil8;
   GLboolean ARB_texture_query_levels;
   GLboolean ARB_texture_query_lod;
   GLboolean ARB_texture_rg;
   GLboolean ARB_texture_rgb10_a2ui;
   GLboolean ARB_texture_view;
   GLboolean ARB_timer_query;
   GLboolean ARB_transform_feedback2;
   GLboolean ARB_transform_feedback3;
   GLboolean ARB_transform_feedback_instanced;
   GLboolean ARB_transform_feedback_overflow_query;
   GLboolean ARB_uniform_buffer_object;
   GLboolean ARB_vertex_attrib_64bit;
   GLboolean ARB_vertex_program;
   GLboolean ARB_vertex_shader;
   GLboolean ARB_vertex_type_10f_11f_11f_rev;
   GLboolean ARB_vertex_type_2_10_10_10_rev;
   GLboolean ARB_viewport_array;
   GLboolean EXT_blend_color;
   GLboolean EXT_blend_equation_separate;
   GLboolean EXT_blend_func_separate;
   GLboolean EXT_blend_minmax;
   GLboolean EXT_demote_to_helper_invocation;
   GLboolean EXT_depth_bounds_test;
   GLboolean EXT_disjoint_timer_query;
   GLboolean EXT_draw_buffers2;
   GLboolean EXT_EGL_image_storage;
   GLboolean EXT_float_blend;
   GLboolean EXT_framebuffer_multisample;
   GLboolean EXT_framebuffer_multisample_blit_scaled;
   GLboolean EXT_framebuffer_sRGB;
   GLboolean EXT_gpu_program_parameters;
   GLboolean EXT_gpu_shader4;
   GLboolean EXT_memory_object;
   GLboolean EXT_memory_object_fd;
   GLboolean EXT_multisampled_render_to_texture;
   GLboolean EXT_packed_float;
   GLboolean EXT_pixel_buffer_object;
   GLboolean EXT_point_parameters;
   GLboolean EXT_provoking_vertex;
   GLboolean EXT_render_snorm;
   GLboolean EXT_semaphore;
   GLboolean EXT_semaphore_fd;
   GLboolean EXT_shader_image_load_formatted;
   GLboolean EXT_shader_image_load_store;
   GLboolean EXT_shader_integer_mix;
   GLboolean EXT_shader_samples_identical;
   GLboolean EXT_sRGB;
   GLboolean EXT_stencil_two_side;
   GLboolean EXT_texture_array;
   GLboolean EXT_texture_buffer_object;
   GLboolean EXT_texture_compression_latc;
   GLboolean EXT_texture_compression_s3tc;
   GLboolean EXT_texture_compression_s3tc_srgb;
   GLboolean EXT_texture_env_dot3;
   GLboolean EXT_texture_filter_anisotropic;
   GLboolean EXT_texture_integer;
   GLboolean EXT_texture_mirror_clamp;
   GLboolean EXT_texture_norm16;
   GLboolean EXT_texture_shadow_lod;
   GLboolean EXT_texture_shared_exponent;
   GLboolean EXT_texture_snorm;
   GLboolean EXT_texture_sRGB;
   GLboolean EXT_texture_sRGB_R8;
   GLboolean EXT_texture_sRGB_decode;
   GLboolean EXT_texture_swizzle;
   GLboolean EXT_texture_type_2_10_10_10_REV;
   GLboolean EXT_transform_feedback;
   GLboolean EXT_timer_query;
   GLboolean EXT_vertex_array_bgra;
   GLboolean EXT_window_rectangles;
   GLboolean OES_copy_image;
   GLboolean OES_primitive_bounding_box;
   GLboolean OES_sample_variables;
   GLboolean OES_standard_derivatives;
   GLboolean OES_texture_buffer;
   GLboolean OES_texture_cube_map_array;
   GLboolean OES_texture_view;
   GLboolean OES_viewport_array;
   /* vendor extensions */
   GLboolean AMD_compressed_ATC_texture;
   GLboolean AMD_framebuffer_multisample_advanced;
   GLboolean AMD_depth_clamp_separate;
   GLboolean AMD_performance_monitor;
   GLboolean AMD_pinned_memory;
   GLboolean AMD_seamless_cubemap_per_texture;
   GLboolean AMD_vertex_shader_layer;
   GLboolean AMD_vertex_shader_viewport_index;
   GLboolean ANDROID_extension_pack_es31a;
   GLboolean APPLE_object_purgeable;
   GLboolean ATI_meminfo;
   GLboolean ATI_texture_compression_3dc;
   GLboolean ATI_texture_mirror_once;
   GLboolean ATI_texture_env_combine3;
   GLboolean ATI_fragment_shader;
   GLboolean GREMEDY_string_marker;
   GLboolean INTEL_blackhole_render;
   GLboolean INTEL_conservative_rasterization;
   GLboolean INTEL_performance_query;
   GLboolean INTEL_shader_atomic_float_minmax;
   GLboolean INTEL_shader_integer_functions2;
   GLboolean KHR_blend_equation_advanced;
   GLboolean KHR_blend_equation_advanced_coherent;
   GLboolean KHR_robustness;
   GLboolean KHR_texture_compression_astc_hdr;
   GLboolean KHR_texture_compression_astc_ldr;
   GLboolean KHR_texture_compression_astc_sliced_3d;
   GLboolean MESA_framebuffer_flip_y;
   GLboolean MESA_tile_raster_order;
   GLboolean MESA_pack_invert;
   GLboolean EXT_shader_framebuffer_fetch;
   GLboolean EXT_shader_framebuffer_fetch_non_coherent;
   GLboolean MESA_shader_integer_functions;
   GLboolean MESA_ycbcr_texture;
   GLboolean NV_alpha_to_coverage_dither_control;
   GLboolean NV_compute_shader_derivatives;
   GLboolean NV_conditional_render;
   GLboolean NV_copy_image;
   GLboolean NV_fill_rectangle;
   GLboolean NV_fog_distance;
   GLboolean NV_point_sprite;
   GLboolean NV_primitive_restart;
   GLboolean NV_shader_atomic_float;
   GLboolean NV_texture_barrier;
   GLboolean NV_texture_env_combine4;
   GLboolean NV_texture_rectangle;
   GLboolean NV_vdpau_interop;
   GLboolean NV_conservative_raster;
   GLboolean NV_conservative_raster_dilate;
   GLboolean NV_conservative_raster_pre_snap_triangles;
   GLboolean NV_conservative_raster_pre_snap;
   GLboolean NV_viewport_array2;
   GLboolean NV_viewport_swizzle;
   GLboolean NVX_gpu_memory_info;
   GLboolean TDFX_texture_compression_FXT1;
   GLboolean OES_EGL_image;
   GLboolean OES_draw_texture;
   GLboolean OES_depth_texture_cube_map;
   GLboolean OES_EGL_image_external;
   GLboolean OES_texture_float;
   GLboolean OES_texture_float_linear;
   GLboolean OES_texture_half_float;
   GLboolean OES_texture_half_float_linear;
   GLboolean OES_compressed_ETC1_RGB8_texture;
   GLboolean OES_geometry_shader;
   GLboolean OES_texture_compression_astc;
   GLboolean extension_sentinel;
   /** The extension string */
   const GLubyte *String;
   /** Number of supported extensions */
   GLuint Count;
   /**
    * The context version which extension helper functions compare against.
    * By default, the value is equal to ctx->Version. This changes to ~0
    * while meta is in progress.
    */
   GLubyte Version;
};


/**
 * A stack of matrices (projection, modelview, color, texture, etc).
 */
struct gl_matrix_stack
{
   GLmatrix *Top;      /**< points into Stack */
   GLmatrix *Stack;    /**< array [MaxDepth] of GLmatrix */
   unsigned StackSize; /**< Number of elements in Stack */
   GLuint Depth;       /**< 0 <= Depth < MaxDepth */
   GLuint MaxDepth;    /**< size of Stack[] array */
   GLuint DirtyFlag;   /**< _NEW_MODELVIEW or _NEW_PROJECTION, for example */
};


/**
 * \name Bits for image transfer operations
 * \sa __struct gl_contextRec::ImageTransferState.
 */
/*@{*/
#define IMAGE_SCALE_BIAS_BIT                      0x1
#define IMAGE_SHIFT_OFFSET_BIT                    0x2
#define IMAGE_MAP_COLOR_BIT                       0x4
#define IMAGE_CLAMP_BIT                           0x800


/** Pixel Transfer ops */
#define IMAGE_BITS (IMAGE_SCALE_BIAS_BIT | \
                    IMAGE_SHIFT_OFFSET_BIT | \
                    IMAGE_MAP_COLOR_BIT)


/**
 * \name Bits to indicate what state has changed.
 */
/*@{*/
#define _NEW_MODELVIEW         (1u << 0)   /**< gl_context::ModelView */
#define _NEW_PROJECTION        (1u << 1)   /**< gl_context::Projection */
#define _NEW_TEXTURE_MATRIX    (1u << 2)   /**< gl_context::TextureMatrix */
#define _NEW_COLOR             (1u << 3)   /**< gl_context::Color */
#define _NEW_DEPTH             (1u << 4)   /**< gl_context::Depth */
/* gap */
#define _NEW_FOG               (1u << 6)   /**< gl_context::Fog */
#define _NEW_HINT              (1u << 7)   /**< gl_context::Hint */
#define _NEW_LIGHT             (1u << 8)   /**< gl_context::Light */
#define _NEW_LINE              (1u << 9)   /**< gl_context::Line */
#define _NEW_PIXEL             (1u << 10)  /**< gl_context::Pixel */
#define _NEW_POINT             (1u << 11)  /**< gl_context::Point */
#define _NEW_POLYGON           (1u << 12)  /**< gl_context::Polygon */
#define _NEW_POLYGONSTIPPLE    (1u << 13)  /**< gl_context::PolygonStipple */
#define _NEW_SCISSOR           (1u << 14)  /**< gl_context::Scissor */
#define _NEW_STENCIL           (1u << 15)  /**< gl_context::Stencil */
#define _NEW_TEXTURE_OBJECT    (1u << 16)  /**< gl_context::Texture (bindings only) */
#define _NEW_TRANSFORM         (1u << 17)  /**< gl_context::Transform */
#define _NEW_VIEWPORT          (1u << 18)  /**< gl_context::Viewport */
#define _NEW_TEXTURE_STATE     (1u << 19)  /**< gl_context::Texture (states only) */
/* gap */
#define _NEW_RENDERMODE        (1u << 21)  /**< gl_context::RenderMode, etc */
#define _NEW_BUFFERS           (1u << 22)  /**< gl_context::Visual, DrawBuffer, */
#define _NEW_CURRENT_ATTRIB    (1u << 23)  /**< gl_context::Current */
#define _NEW_MULTISAMPLE       (1u << 24)  /**< gl_context::Multisample */
#define _NEW_TRACK_MATRIX      (1u << 25)  /**< gl_context::VertexProgram */
#define _NEW_PROGRAM           (1u << 26)  /**< New program/shader state */
#define _NEW_PROGRAM_CONSTANTS (1u << 27)
/* gap */
#define _NEW_FRAG_CLAMP        (1u << 29)
/* gap, re-use for core Mesa state only; use ctx->DriverFlags otherwise */
#define _NEW_VARYING_VP_INPUTS (1u << 31) /**< gl_context::varying_vp_inputs */
#define _NEW_ALL ~0
/*@}*/


/**
 * Composite state flags
 */
/*@{*/
#define _NEW_TEXTURE   (_NEW_TEXTURE_OBJECT | _NEW_TEXTURE_STATE)

#define _MESA_NEW_NEED_EYE_COORDS         (_NEW_LIGHT |		\
                                           _NEW_TEXTURE_STATE |	\
                                           _NEW_POINT |		\
                                           _NEW_PROGRAM |	\
                                           _NEW_MODELVIEW)

#define _MESA_NEW_SEPARATE_SPECULAR        (_NEW_LIGHT | \
                                            _NEW_FOG | \
                                            _NEW_PROGRAM)


/*@}*/




/* This has to be included here. */
#include "dd.h"


/** Opaque declaration of display list payload data type */
union gl_dlist_node;


/**
 * Per-display list information.
 */
struct gl_display_list
{
   GLuint Name;
   GLbitfield Flags;  /**< DLIST_x flags */
   GLchar *Label;     /**< GL_KHR_debug */
   /** The dlist commands are in a linked list of nodes */
   union gl_dlist_node *Head;
};


/**
 * State used during display list compilation and execution.
 */
struct gl_dlist_state
{
   struct gl_display_list *CurrentList; /**< List currently being compiled */
   union gl_dlist_node *CurrentBlock; /**< Pointer to current block of nodes */
   GLuint CurrentPos;		/**< Index into current block of nodes */
   GLuint CallDepth;		/**< Current recursion calling depth */

   GLvertexformat ListVtxfmt;

   GLubyte ActiveAttribSize[VERT_ATTRIB_MAX];
   uint32_t CurrentAttrib[VERT_ATTRIB_MAX][8];

   GLubyte ActiveMaterialSize[MAT_ATTRIB_MAX];
   GLfloat CurrentMaterial[MAT_ATTRIB_MAX][4];

   struct {
      /* State known to have been set by the currently-compiling display
       * list.  Used to eliminate some redundant state changes.
       */
      GLenum16 ShadeModel;
   } Current;
};

/**
 * Driver-specific state flags.
 *
 * These are or'd with gl_context::NewDriverState to notify a driver about
 * a state change. The driver sets the flags at context creation and
 * the meaning of the bits set is opaque to core Mesa.
 */
struct gl_driver_flags
{
   /** gl_context::Array::_DrawArrays (vertex array state) */
   uint64_t NewArray;

   /** gl_context::TransformFeedback::CurrentObject */
   uint64_t NewTransformFeedback;

   /** gl_context::TransformFeedback::CurrentObject::shader_program */
   uint64_t NewTransformFeedbackProg;

   /** gl_context::RasterDiscard */
   uint64_t NewRasterizerDiscard;

   /** gl_context::TileRasterOrder* */
   uint64_t NewTileRasterOrder;

   /**
    * gl_context::UniformBufferBindings
    * gl_shader_program::UniformBlocks
    */
   uint64_t NewUniformBuffer;

   /**
    * gl_context::ShaderStorageBufferBindings
    * gl_shader_program::ShaderStorageBlocks
    */
   uint64_t NewShaderStorageBuffer;

   uint64_t NewTextureBuffer;

   /**
    * gl_context::AtomicBufferBindings
    */
   uint64_t NewAtomicBuffer;

   /**
    * gl_context::ImageUnits
    */
   uint64_t NewImageUnits;

   /**
    * gl_context::TessCtrlProgram::patch_default_*
    */
   uint64_t NewDefaultTessLevels;

   /**
    * gl_context::IntelConservativeRasterization
    */
   uint64_t NewIntelConservativeRasterization;

   /**
    * gl_context::NvConservativeRasterization
    */
   uint64_t NewNvConservativeRasterization;

   /**
    * gl_context::ConservativeRasterMode/ConservativeRasterDilate
    * gl_context::SubpixelPrecisionBias
    */
   uint64_t NewNvConservativeRasterizationParams;

   /**
    * gl_context::Scissor::WindowRects
    */
   uint64_t NewWindowRectangles;

   /** gl_context::Color::sRGBEnabled */
   uint64_t NewFramebufferSRGB;

   /** gl_context::Scissor::EnableFlags */
   uint64_t NewScissorTest;

   /** gl_context::Scissor::ScissorArray */
   uint64_t NewScissorRect;

   /** gl_context::Color::Alpha* */
   uint64_t NewAlphaTest;

   /** gl_context::Color::Blend/Dither */
   uint64_t NewBlend;

   /** gl_context::Color::BlendColor */
   uint64_t NewBlendColor;

   /** gl_context::Color::Color/Index */
   uint64_t NewColorMask;

   /** gl_context::Depth */
   uint64_t NewDepth;

   /** gl_context::Color::LogicOp/ColorLogicOp/IndexLogicOp */
   uint64_t NewLogicOp;

   /** gl_context::Multisample::Enabled */
   uint64_t NewMultisampleEnable;

   /** gl_context::Multisample::SampleAlphaTo* */
   uint64_t NewSampleAlphaToXEnable;

   /** gl_context::Multisample::SampleCoverage/SampleMaskValue */
   uint64_t NewSampleMask;

   /** gl_context::Multisample::(Min)SampleShading */
   uint64_t NewSampleShading;

   /** gl_context::Stencil */
   uint64_t NewStencil;

   /** gl_context::Transform::ClipOrigin/ClipDepthMode */
   uint64_t NewClipControl;

   /** gl_context::Transform::EyeUserPlane */
   uint64_t NewClipPlane;

   /** gl_context::Transform::ClipPlanesEnabled */
   uint64_t NewClipPlaneEnable;

   /** gl_context::Transform::DepthClamp */
   uint64_t NewDepthClamp;

   /** gl_context::Line */
   uint64_t NewLineState;

   /** gl_context::Polygon */
   uint64_t NewPolygonState;

   /** gl_context::PolygonStipple */
   uint64_t NewPolygonStipple;

   /** gl_context::ViewportArray */
   uint64_t NewViewport;

   /** Shader constants (uniforms, program parameters, state constants) */
   uint64_t NewShaderConstants[MESA_SHADER_STAGES];

   /** Programmable sample location state for gl_context::DrawBuffer */
   uint64_t NewSampleLocations;
};

struct gl_buffer_binding
{
   struct gl_buffer_object *BufferObject;
   /** Start of uniform block data in the buffer */
   GLintptr Offset;
   /** Size of data allowed to be referenced from the buffer (in bytes) */
   GLsizeiptr Size;
   /**
    * glBindBufferBase() indicates that the Size should be ignored and only
    * limited by the current size of the BufferObject.
    */
   GLboolean AutomaticSize;
};

/**
 * ARB_shader_image_load_store image unit.
 */
struct gl_image_unit
{
   /**
    * Texture object bound to this unit.
    */
   struct gl_texture_object *TexObj;

   /**
    * Level of the texture object bound to this unit.
    */
   GLubyte Level;

   /**
    * \c GL_TRUE if the whole level is bound as an array of layers, \c
    * GL_FALSE if only some specific layer of the texture is bound.
    * \sa Layer
    */
   GLboolean Layered;

   /**
    * Layer of the texture object bound to this unit as specified by the
    * application.
    */
   GLushort Layer;

   /**
    * Layer of the texture object bound to this unit, or zero if
    * Layered == false.
    */
   GLushort _Layer;

   /**
    * Access allowed to this texture image.  Either \c GL_READ_ONLY,
    * \c GL_WRITE_ONLY or \c GL_READ_WRITE.
    */
   GLenum16 Access;

   /**
    * GL internal format that determines the interpretation of the
    * image memory when shader image operations are performed through
    * this unit.
    */
   GLenum16 Format;

   /**
    * Mesa format corresponding to \c Format.
    */
   mesa_format _ActualFormat:16;
};

/**
 * Shader subroutines storage
 */
struct gl_subroutine_index_binding
{
   GLuint NumIndex;
   GLuint *IndexPtr;
};

struct gl_texture_handle_object
{
   struct gl_texture_object *texObj;
   struct gl_sampler_object *sampObj;
   GLuint64 handle;
};

struct gl_image_handle_object
{
   struct gl_image_unit imgObj;
   GLuint64 handle;
};

struct gl_memory_object
{
   GLuint Name;            /**< hash table ID/name */
   GLboolean Immutable;    /**< denotes mutability state of parameters */
   GLboolean Dedicated;    /**< import memory from a dedicated allocation */
};

struct gl_semaphore_object
{
   GLuint Name;            /**< hash table ID/name */
};

/**
 * Mesa rendering context.
 *
 * This is the central context data structure for Mesa.  Almost all
 * OpenGL state is contained in this structure.
 * Think of this as a base class from which device drivers will derive
 * sub classes.
 */
struct gl_context
{
   /** State possibly shared with other contexts in the address space */
   struct gl_shared_state *Shared;

   /** \name API function pointer tables */
   /*@{*/
   gl_api API;

   /**
    * The current dispatch table for non-displaylist-saving execution, either
    * BeginEnd or OutsideBeginEnd
    */
   struct _glapi_table *Exec;
   /**
    * The normal dispatch table for non-displaylist-saving, non-begin/end
    */
   struct _glapi_table *OutsideBeginEnd;
   /** The dispatch table used between glNewList() and glEndList() */
   struct _glapi_table *Save;
   /**
    * The dispatch table used between glBegin() and glEnd() (outside of a
    * display list).  Only valid functions between those two are set, which is
    * mostly just the set in a GLvertexformat struct.
    */
   struct _glapi_table *BeginEnd;
   /**
    * Dispatch table for when a graphics reset has happened.
    */
   struct _glapi_table *ContextLost;
   /**
    * Dispatch table used to marshal API calls from the client program to a
    * separate server thread.  NULL if API calls are not being marshalled to
    * another thread.
    */
   struct _glapi_table *MarshalExec;
   /**
    * Dispatch table currently in use for fielding API calls from the client
    * program.  If API calls are being marshalled to another thread, this ==
    * MarshalExec.  Otherwise it == CurrentServerDispatch.
    */
   struct _glapi_table *CurrentClientDispatch;

   /**
    * Dispatch table currently in use for performing API calls.  == Save or
    * Exec.
    */
   struct _glapi_table *CurrentServerDispatch;

   /*@}*/

   struct glthread_state GLThread;

   struct gl_config Visual;
   struct gl_framebuffer *DrawBuffer;	/**< buffer for writing */
   struct gl_framebuffer *ReadBuffer;	/**< buffer for reading */
   struct gl_framebuffer *WinSysDrawBuffer;  /**< set with MakeCurrent */
   struct gl_framebuffer *WinSysReadBuffer;  /**< set with MakeCurrent */

   /**
    * Device driver function pointer table
    */
   struct dd_function_table Driver;

   /** Core/Driver constants */
   struct gl_constants Const;

   /** \name The various 4x4 matrix stacks */
   /*@{*/
   struct gl_matrix_stack ModelviewMatrixStack;
   struct gl_matrix_stack ProjectionMatrixStack;
   struct gl_matrix_stack TextureMatrixStack[MAX_TEXTURE_UNITS];
   struct gl_matrix_stack ProgramMatrixStack[MAX_PROGRAM_MATRICES];
   struct gl_matrix_stack *CurrentStack; /**< Points to one of the above stacks */
   /*@}*/

   /** Combined modelview and projection matrix */
   GLmatrix _ModelProjectMatrix;

   /** \name Display lists */
   struct gl_dlist_state ListState;

   GLboolean ExecuteFlag;	/**< Execute GL commands? */
   GLboolean CompileFlag;	/**< Compile GL commands into display list? */

   /** Extension information */
   struct gl_extensions Extensions;

   /** GL version integer, for example 31 for GL 3.1, or 20 for GLES 2.0. */
   GLuint Version;
   char *VersionString;

   /** \name State attribute stack (for glPush/PopAttrib) */
   /*@{*/
   GLuint AttribStackDepth;
   struct gl_attrib_node *AttribStack[MAX_ATTRIB_STACK_DEPTH];
   /*@}*/

   /** \name Renderer attribute groups
    *
    * We define a struct for each attribute group to make pushing and popping
    * attributes easy.  Also it's a good organization.
    */
   /*@{*/
   struct gl_accum_attrib	Accum;		/**< Accum buffer attributes */
   struct gl_colorbuffer_attrib	Color;		/**< Color buffer attributes */
   struct gl_current_attrib	Current;	/**< Current attributes */
   struct gl_depthbuffer_attrib	Depth;		/**< Depth buffer attributes */
   struct gl_eval_attrib	Eval;		/**< Eval attributes */
   struct gl_fog_attrib		Fog;		/**< Fog attributes */
   struct gl_hint_attrib	Hint;		/**< Hint attributes */
   struct gl_light_attrib	Light;		/**< Light attributes */
   struct gl_line_attrib	Line;		/**< Line attributes */
   struct gl_list_attrib	List;		/**< List attributes */
   struct gl_multisample_attrib Multisample;
   struct gl_pixel_attrib	Pixel;		/**< Pixel attributes */
   struct gl_point_attrib	Point;		/**< Point attributes */
   struct gl_polygon_attrib	Polygon;	/**< Polygon attributes */
   GLuint PolygonStipple[32];			/**< Polygon stipple */
   struct gl_scissor_attrib	Scissor;	/**< Scissor attributes */
   struct gl_stencil_attrib	Stencil;	/**< Stencil buffer attributes */
   struct gl_texture_attrib	Texture;	/**< Texture attributes */
   struct gl_transform_attrib	Transform;	/**< Transformation attributes */
   struct gl_viewport_attrib	ViewportArray[MAX_VIEWPORTS];	/**< Viewport attributes */
   GLuint SubpixelPrecisionBias[2];	/**< Viewport attributes */
   /*@}*/

   /** \name Client attribute stack */
   /*@{*/
   GLuint ClientAttribStackDepth;
   struct gl_attrib_node *ClientAttribStack[MAX_CLIENT_ATTRIB_STACK_DEPTH];
   /*@}*/

   /** \name Client attribute groups */
   /*@{*/
   struct gl_array_attrib	Array;	/**< Vertex arrays */
   struct gl_pixelstore_attrib	Pack;	/**< Pixel packing */
   struct gl_pixelstore_attrib	Unpack;	/**< Pixel unpacking */
   struct gl_pixelstore_attrib	DefaultPacking;	/**< Default params */
   /*@}*/

   /** \name Other assorted state (not pushed/popped on attribute stack) */
   /*@{*/
   struct gl_pixelmaps          PixelMaps;

   struct gl_evaluators EvalMap;   /**< All evaluators */
   struct gl_feedback   Feedback;  /**< Feedback */
   struct gl_selection  Select;    /**< Selection */

   struct gl_program_state Program;  /**< general program state */
   struct gl_vertex_program_state VertexProgram;
   struct gl_fragment_program_state FragmentProgram;
   struct gl_geometry_program_state GeometryProgram;
   struct gl_compute_program_state ComputeProgram;
   struct gl_tess_ctrl_program_state TessCtrlProgram;
   struct gl_tess_eval_program_state TessEvalProgram;
   struct gl_ati_fragment_shader_state ATIFragmentShader;

   struct gl_pipeline_shader_state Pipeline; /**< GLSL pipeline shader object state */
   struct gl_pipeline_object Shader; /**< GLSL shader object state */

   /**
    * Current active shader pipeline state
    *
    * Almost all internal users want ::_Shader instead of ::Shader.  The
    * exceptions are bits of legacy GLSL API that do not know about separate
    * shader objects.
    *
    * If a program is active via \c glUseProgram, this will point to
    * \c ::Shader.
    *
    * If a program pipeline is active via \c glBindProgramPipeline, this will
    * point to \c ::Pipeline.Current.
    *
    * If neither a program nor a program pipeline is active, this will point to
    * \c ::Pipeline.Default.  This ensures that \c ::_Shader will never be
    * \c NULL.
    */
   struct gl_pipeline_object *_Shader;

   /**
    * NIR containing the functions that implement software fp64 support.
    */
   struct nir_shader *SoftFP64;

   struct gl_query_state Query;  /**< occlusion, timer queries */

   struct gl_transform_feedback_state TransformFeedback;

   struct gl_perf_monitor_state PerfMonitor;
   struct gl_perf_query_state PerfQuery;

   struct gl_buffer_object *DrawIndirectBuffer; /** < GL_ARB_draw_indirect */
   struct gl_buffer_object *ParameterBuffer; /** < GL_ARB_indirect_parameters */
   struct gl_buffer_object *DispatchIndirectBuffer; /** < GL_ARB_compute_shader */

   struct gl_buffer_object *CopyReadBuffer; /**< GL_ARB_copy_buffer */
   struct gl_buffer_object *CopyWriteBuffer; /**< GL_ARB_copy_buffer */

   struct gl_buffer_object *QueryBuffer; /**< GL_ARB_query_buffer_object */

   /**
    * Current GL_ARB_uniform_buffer_object binding referenced by
    * GL_UNIFORM_BUFFER target for glBufferData, glMapBuffer, etc.
    */
   struct gl_buffer_object *UniformBuffer;

   /**
    * Current GL_ARB_shader_storage_buffer_object binding referenced by
    * GL_SHADER_STORAGE_BUFFER target for glBufferData, glMapBuffer, etc.
    */
   struct gl_buffer_object *ShaderStorageBuffer;

   /**
    * Array of uniform buffers for GL_ARB_uniform_buffer_object and GL 3.1.
    * This is set up using glBindBufferRange() or glBindBufferBase().  They are
    * associated with uniform blocks by glUniformBlockBinding()'s state in the
    * shader program.
    */
   struct gl_buffer_binding
      UniformBufferBindings[MAX_COMBINED_UNIFORM_BUFFERS];

   /**
    * Array of shader storage buffers for ARB_shader_storage_buffer_object
    * and GL 4.3. This is set up using glBindBufferRange() or
    * glBindBufferBase().  They are associated with shader storage blocks by
    * glShaderStorageBlockBinding()'s state in the shader program.
    */
   struct gl_buffer_binding
      ShaderStorageBufferBindings[MAX_COMBINED_SHADER_STORAGE_BUFFERS];

   /**
    * Object currently associated with the GL_ATOMIC_COUNTER_BUFFER
    * target.
    */
   struct gl_buffer_object *AtomicBuffer;

   /**
    * Object currently associated w/ the GL_EXTERNAL_VIRTUAL_MEMORY_BUFFER_AMD
    * target.
    */
   struct gl_buffer_object *ExternalVirtualMemoryBuffer;

   /**
    * Array of atomic counter buffer binding points.
    */
   struct gl_buffer_binding
      AtomicBufferBindings[MAX_COMBINED_ATOMIC_BUFFERS];

   /**
    * Array of image units for ARB_shader_image_load_store.
    */
   struct gl_image_unit ImageUnits[MAX_IMAGE_UNITS];

   struct gl_subroutine_index_binding SubroutineIndex[MESA_SHADER_STAGES];
   /*@}*/

   struct gl_meta_state *Meta;  /**< for "meta" operations */

   /* GL_EXT_framebuffer_object */
   struct gl_renderbuffer *CurrentRenderbuffer;

   GLenum16 ErrorValue;      /**< Last error code */

   /**
    * Recognize and silence repeated error debug messages in buggy apps.
    */
   const char *ErrorDebugFmtString;
   GLuint ErrorDebugCount;

   /* GL_ARB_debug_output/GL_KHR_debug */
   simple_mtx_t DebugMutex;
   struct gl_debug_state *Debug;

   GLenum16 RenderMode;      /**< either GL_RENDER, GL_SELECT, GL_FEEDBACK */
   GLbitfield NewState;      /**< bitwise-or of _NEW_* flags */
   uint64_t NewDriverState;  /**< bitwise-or of flags from DriverFlags */

   struct gl_driver_flags DriverFlags;

   GLboolean ViewportInitialized;  /**< has viewport size been initialized? */
   GLboolean _AllowDrawOutOfOrder;

   GLbitfield varying_vp_inputs;  /**< mask of VERT_BIT_* flags */

   /** \name Derived state */
   GLbitfield _ImageTransferState;/**< bitwise-or of IMAGE_*_BIT flags */
   GLfloat _EyeZDir[3];
   GLfloat _ModelViewInvScale; /* may be for model- or eyespace lighting */
   GLfloat _ModelViewInvScaleEyespace; /* always factor defined in spec */
   GLboolean _NeedEyeCoords;
   GLboolean _ForceEyeCoords;

   GLuint TextureStateTimestamp; /**< detect changes to shared state */

   struct gl_list_extensions *ListExt; /**< driver dlist extensions */

   /** \name For debugging/development only */
   /*@{*/
   GLboolean FirstTimeCurrent;
   /*@}*/

   /**
    * False if this context was created without a config. This is needed
    * because the initial state of glDrawBuffers depends on this
    */
   GLboolean HasConfig;

   GLboolean TextureFormatSupported[MESA_FORMAT_COUNT];

   GLboolean RasterDiscard;  /**< GL_RASTERIZER_DISCARD */
   GLboolean IntelConservativeRasterization; /**< GL_CONSERVATIVE_RASTERIZATION_INTEL */
   GLboolean ConservativeRasterization; /**< GL_CONSERVATIVE_RASTERIZATION_NV */
   GLfloat ConservativeRasterDilate;
   GLenum16 ConservativeRasterMode;

   GLboolean IntelBlackholeRender; /**< GL_INTEL_blackhole_render */

   /** Does glVertexAttrib(0) alias glVertex()? */
   bool _AttribZeroAliasesVertex;

   /**
    * When set, TileRasterOrderIncreasingX/Y control the order that a tiled
    * renderer's tiles should be excecuted, to meet the requirements of
    * GL_MESA_tile_raster_order.
    */
   GLboolean TileRasterOrderFixed;
   GLboolean TileRasterOrderIncreasingX;
   GLboolean TileRasterOrderIncreasingY;

   /**
    * \name Hooks for module contexts.
    *
    * These will eventually live in the driver or elsewhere.
    */
   /*@{*/
   void *swrast_context;
   void *swsetup_context;
   void *swtnl_context;
   struct vbo_context *vbo_context;
   struct st_context *st;
   /*@}*/

   /**
    * \name NV_vdpau_interop
    */
   /*@{*/
   const void *vdpDevice;
   const void *vdpGetProcAddress;
   struct set *vdpSurfaces;
   /*@}*/

   /**
    * Has this context observed a GPU reset in any context in the share group?
    *
    * Once this field becomes true, it is never reset to false.
    */
   GLboolean ShareGroupReset;

   /**
    * \name OES_primitive_bounding_box
    *
    * Stores the arguments to glPrimitiveBoundingBox
    */
   GLfloat PrimitiveBoundingBox[8];

   struct disk_cache *Cache;

   /**
    * \name GL_ARB_bindless_texture
    */
   /*@{*/
   struct hash_table_u64 *ResidentTextureHandles;
   struct hash_table_u64 *ResidentImageHandles;
   /*@}*/

   bool shader_builtin_ref;
};

/**
 * Information about memory usage. All sizes are in kilobytes.
 */
struct gl_memory_info
{
   unsigned total_device_memory; /**< size of device memory, e.g. VRAM */
   unsigned avail_device_memory; /**< free device memory at the moment */
   unsigned total_staging_memory; /**< size of staging memory, e.g. GART */
   unsigned avail_staging_memory; /**< free staging memory at the moment */
   unsigned device_memory_evicted; /**< size of memory evicted (monotonic counter) */
   unsigned nr_device_memory_evictions; /**< # of evictions (monotonic counter) */
};

#ifndef NDEBUG
extern int MESA_VERBOSE;
extern int MESA_DEBUG_FLAGS;
#else
# define MESA_VERBOSE 0
# define MESA_DEBUG_FLAGS 0
#endif


/** The MESA_VERBOSE var is a bitmask of these flags */
enum _verbose
{
   VERBOSE_VARRAY		= 0x0001,
   VERBOSE_TEXTURE		= 0x0002,
   VERBOSE_MATERIAL		= 0x0004,
   VERBOSE_PIPELINE		= 0x0008,
   VERBOSE_DRIVER		= 0x0010,
   VERBOSE_STATE		= 0x0020,
   VERBOSE_API			= 0x0040,
   VERBOSE_DISPLAY_LIST		= 0x0100,
   VERBOSE_LIGHTING		= 0x0200,
   VERBOSE_PRIMS		= 0x0400,
   VERBOSE_VERTS		= 0x0800,
   VERBOSE_DISASSEM		= 0x1000,
   VERBOSE_DRAW                 = 0x2000,
   VERBOSE_SWAPBUFFERS          = 0x4000
};


/** The MESA_DEBUG_FLAGS var is a bitmask of these flags */
enum _debug
{
   DEBUG_SILENT                 = (1 << 0),
   DEBUG_ALWAYS_FLUSH		= (1 << 1),
   DEBUG_INCOMPLETE_TEXTURE     = (1 << 2),
   DEBUG_INCOMPLETE_FBO         = (1 << 3),
   DEBUG_CONTEXT                = (1 << 4)
};

#ifdef __cplusplus
}
#endif

#endif /* MTYPES_H */
