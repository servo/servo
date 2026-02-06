/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 1999-2005  Brian Paul   All Rights Reserved.
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
 * \file math/m_matrix.h
 * Defines basic structures for matrix-handling.
 */

#ifndef _M_MATRIX_H
#define _M_MATRIX_H


#include "main/glheader.h"


#ifdef __cplusplus
extern "C" {
#endif


/**
 * \name Symbolic names to some of the entries in the matrix
 *
 * These are handy for the viewport mapping, which is expressed as a matrix.
 */
/*@{*/
#define MAT_SX 0
#define MAT_SY 5
#define MAT_SZ 10
#define MAT_TX 12
#define MAT_TY 13
#define MAT_TZ 14
/*@}*/


/**
 * Different kinds of 4x4 transformation matrices.
 * We use these to select specific optimized vertex transformation routines.
 */
enum GLmatrixtype {
   MATRIX_GENERAL,	/**< general 4x4 matrix */
   MATRIX_IDENTITY,	/**< identity matrix */
   MATRIX_3D_NO_ROT,	/**< orthogonal projection and others... */
   MATRIX_PERSPECTIVE,	/**< perspective projection matrix */
   MATRIX_2D,		/**< 2-D transformation */
   MATRIX_2D_NO_ROT,	/**< 2-D scale & translate only */
   MATRIX_3D		/**< 3-D transformation */
} ;

/**
 * Matrix type to represent 4x4 transformation matrices.
 */
typedef struct {
   GLfloat *m;		/**< 16 matrix elements (16-byte aligned) */
   GLfloat *inv;	/**< 16-element inverse (16-byte aligned) */
   GLuint flags;        /**< possible values determined by (of \link
                         * MatFlags MAT_FLAG_* flags\endlink)
                         */
   enum GLmatrixtype type;
} GLmatrix;




extern void
_math_matrix_ctr( GLmatrix *m );

extern void
_math_matrix_dtr( GLmatrix *m );

extern void
_math_matrix_mul_matrix( GLmatrix *dest, const GLmatrix *a, const GLmatrix *b );

extern void
_math_matrix_mul_floats( GLmatrix *dest, const GLfloat *b );

extern void
_math_matrix_loadf( GLmatrix *mat, const GLfloat *m );

extern void
_math_matrix_translate( GLmatrix *mat, GLfloat x, GLfloat y, GLfloat z );

extern void
_math_matrix_rotate( GLmatrix *m, GLfloat angle,
		     GLfloat x, GLfloat y, GLfloat z );

extern void
_math_matrix_scale( GLmatrix *mat, GLfloat x, GLfloat y, GLfloat z );

extern void
_math_matrix_ortho( GLmatrix *mat,
		    GLfloat left, GLfloat right,
		    GLfloat bottom, GLfloat top,
		    GLfloat nearval, GLfloat farval );

extern void
_math_matrix_frustum( GLmatrix *mat,
		      GLfloat left, GLfloat right,
		      GLfloat bottom, GLfloat top,
		      GLfloat nearval, GLfloat farval );

extern void
_math_matrix_viewport( GLmatrix *m, const float scale[3],
                       const float translate[3], double depthMax );

extern void
_math_matrix_set_identity( GLmatrix *dest );

extern void
_math_matrix_copy( GLmatrix *to, const GLmatrix *from );

extern void
_math_matrix_analyse( GLmatrix *mat );

extern void
_math_matrix_print( const GLmatrix *m );

extern GLboolean
_math_matrix_is_length_preserving( const GLmatrix *m );

extern GLboolean
_math_matrix_has_rotation( const GLmatrix *m );

extern GLboolean
_math_matrix_is_general_scale( const GLmatrix *m );

extern GLboolean
_math_matrix_is_dirty( const GLmatrix *m );


/**
 * \name Related functions that don't actually operate on GLmatrix structs
 */
/*@{*/

extern void
_math_transposef( GLfloat to[16], const GLfloat from[16] );

extern void
_math_transposed( GLdouble to[16], const GLdouble from[16] );

extern void
_math_transposefd( GLfloat to[16], const GLdouble from[16] );


/*
 * Transform a point (column vector) by a matrix:   Q = M * P
 */
#define TRANSFORM_POINT( Q, M, P )					\
   Q[0] = M[0] * P[0] + M[4] * P[1] + M[8] *  P[2] + M[12] * P[3];	\
   Q[1] = M[1] * P[0] + M[5] * P[1] + M[9] *  P[2] + M[13] * P[3];	\
   Q[2] = M[2] * P[0] + M[6] * P[1] + M[10] * P[2] + M[14] * P[3];	\
   Q[3] = M[3] * P[0] + M[7] * P[1] + M[11] * P[2] + M[15] * P[3];


#define TRANSFORM_POINT3( Q, M, P )				\
   Q[0] = M[0] * P[0] + M[4] * P[1] + M[8] *  P[2] + M[12];	\
   Q[1] = M[1] * P[0] + M[5] * P[1] + M[9] *  P[2] + M[13];	\
   Q[2] = M[2] * P[0] + M[6] * P[1] + M[10] * P[2] + M[14];	\
   Q[3] = M[3] * P[0] + M[7] * P[1] + M[11] * P[2] + M[15];


/*
 * Transform a normal (row vector) by a matrix:  [NX NY NZ] = N * MAT
 */
#define TRANSFORM_NORMAL( TO, N, MAT )				\
do {								\
   TO[0] = N[0] * MAT[0] + N[1] * MAT[1] + N[2] * MAT[2];	\
   TO[1] = N[0] * MAT[4] + N[1] * MAT[5] + N[2] * MAT[6];	\
   TO[2] = N[0] * MAT[8] + N[1] * MAT[9] + N[2] * MAT[10];	\
} while (0)


/**
 * Transform a direction by a matrix.
 */
#define TRANSFORM_DIRECTION( TO, DIR, MAT )			\
do {								\
   TO[0] = DIR[0] * MAT[0] + DIR[1] * MAT[4] + DIR[2] * MAT[8];	\
   TO[1] = DIR[0] * MAT[1] + DIR[1] * MAT[5] + DIR[2] * MAT[9];	\
   TO[2] = DIR[0] * MAT[2] + DIR[1] * MAT[6] + DIR[2] * MAT[10];\
} while (0)


extern void
_mesa_transform_vector(GLfloat u[4], const GLfloat v[4], const GLfloat m[16]);


/*@}*/


#ifdef __cplusplus
}
#endif

#endif
