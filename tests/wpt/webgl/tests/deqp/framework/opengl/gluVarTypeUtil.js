/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('framework.opengl.gluVarTypeUtil');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluVarType');

goog.scope(function() {

    var gluVarTypeUtil = framework.opengl.gluVarTypeUtil;
    var gluVarType = framework.opengl.gluVarType;
    var gluShaderUtil = framework.opengl.gluShaderUtil;

    gluVarTypeUtil.isNum = function(c) { return /^[0-9]$/.test(c); };
    gluVarTypeUtil.isAlpha = function(c) { return /^[a-zA-Z]$/.test(c); };
    gluVarTypeUtil.isIdentifierChar = function(c) { return /^[a-zA-Z0-9_]$/.test(c); };
    gluVarTypeUtil.array_op_equivalent = function(arr1, arr2) {
        if (arr1.length != arr2.length) return false;
        for (var i = 0; i < arr1.length; ++i) {
            if (arr1[i].isnt(arr2[1])) return false;
        }
        return true;
    };

    /**
     * gluVarTypeUtil.VarTokenizer class.
     * @param {string} str
     * @constructor
     */
    gluVarTypeUtil.VarTokenizer = function(str) {

        /** @private */
        this.m_str = str;
        /** @private */
        this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.length;
        /** @private */
        this.m_tokenStart = 0;
        /** @private */
        this.m_tokenLen = 0;

        this.advance();

    };
    gluVarTypeUtil.VarTokenizer.s_Token = {
        IDENTIFIER: 0,
        LEFT_BRACKET: 1,
        RIGHT_BRACKET: 2,
        PERIOD: 3,
        NUMBER: 4,
        END: 5
    };
    gluVarTypeUtil.VarTokenizer.s_Token.length = Object.keys(gluVarTypeUtil.VarTokenizer.s_Token).length;

    gluVarTypeUtil.VarTokenizer.prototype.getToken = function() {
        return this.m_token;
    };
    gluVarTypeUtil.VarTokenizer.prototype.getIdentifier = function() {
        return this.m_str.substr(this.m_tokenStart, this.m_tokenLen);
    };
    gluVarTypeUtil.VarTokenizer.prototype.getNumber = function() {
        return parseInt(this.getIdentifier(), 10);
    };
    gluVarTypeUtil.VarTokenizer.prototype.getCurrentTokenStartLocation = function() {
        return this.m_tokenStart;
    };
    gluVarTypeUtil.VarTokenizer.prototype.getCurrentTokenEndLocation = function() {
        return this.m_tokenStart + this.m_tokenLen;
    };

    gluVarTypeUtil.VarTokenizer.prototype.advance = function() {

        if (this.m_token == gluVarTypeUtil.VarTokenizer.s_Token.END) {
            throw new Error('No more tokens.');
        }

        this.m_tokenStart += this.m_tokenLen;
        this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.length;
        this.m_tokenLen = 1;

        if (this.m_tokenStart >= this.m_str.length) {
            this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.END;

        } else if (this.m_str[this.m_tokenStart] == '[') {
            this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.LEFT_BRACKET;

        } else if (this.m_str[this.m_tokenStart] == ']') {
            this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.RIGHT_BRACKET;

        } else if (this.m_str[this.m_tokenStart] == '.') {
            this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.PERIOD;

        } else if (gluVarTypeUtil.isNum(this.m_str[this.m_tokenStart])) {
            this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.NUMBER;
            while (gluVarTypeUtil.isNum(this.m_str[this.m_tokenStart + this.m_tokenLen])) {
                this.m_tokenLen += 1;
            }

        } else if (gluVarTypeUtil.isIdentifierChar(this.m_str[this.m_tokenStart])) {
            this.m_token = gluVarTypeUtil.VarTokenizer.s_Token.IDENTIFIER;
            while (gluVarTypeUtil.isIdentifierChar(this.m_str[this.m_tokenStart + this.m_tokenLen])) {
                this.m_tokenLen += 1;
            }

        } else {
            throw new Error('Unexpected character');
        }

    };

    /**
     * VarType subtype path utilities class.
     * @param {gluVarTypeUtil.VarTypeComponent.s_Type} type
     * @param {number} index
     * @constructor
     */
    gluVarTypeUtil.VarTypeComponent = function(type, index) {
        /** @type {gluVarTypeUtil.VarTypeComponent.s_Type} */ this.type = type;
        this.index = index || 0;
    };

    gluVarTypeUtil.VarTypeComponent.prototype.is = function(other) {
        return this.type == other.type && this.index == other.index;
    };
    gluVarTypeUtil.VarTypeComponent.prototype.isnt = function(other) {
        return this.type != other.type || this.index != other.index;
    };

    /**
     * @enum
     */
    gluVarTypeUtil.VarTypeComponent.s_Type = {
        STRUCT_MEMBER: 0,
        ARRAY_ELEMENT: 1,
        MATRIX_COLUMN: 2,
        VECTOR_COMPONENT: 3
    };

    /**
     * Type path formatter.
     * @param {gluVarType.VarType} type_
     * @param {Array<gluVarTypeUtil.VarTypeComponent>} path_
     * @constructor
     */
    gluVarTypeUtil.TypeAccessFormat = function(type_, path_) {
        this.type = type_;
        this.path = path_;
    };

    gluVarTypeUtil.TypeAccessFormat.prototype.toString = function() {
        var curType = this.type;
        var str = '';

        for (var i = 0; i < this.path.length; i++) {
            var iter = this.path[i];
            switch (iter.type) {
                case gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT:
                    curType = curType.getElementType(); // Update current type.
                    // Fall-through.

                case gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN:
                case gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT:
                    str += '[' + iter.index + ']';
                    break;

                case gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER: {
                    var member = curType.getStruct().getMember(i);
                    str += '.' + member.getName();
                    curType = member.getType();
                    break;
                }

                default:
                   throw new Error('Unrecognized type:' + iter.type);
            }
        }

        return str;
    };

    /** gluVarTypeUtil.SubTypeAccess
     * @param {gluVarType.VarType} type
     * @constructor
     */
    gluVarTypeUtil.SubTypeAccess = function(type) {

        this.m_type = null; // VarType
        this.m_path = []; // TypeComponentVector

    };

    /** @private */
    gluVarTypeUtil.SubTypeAccess.prototype.helper = function(type, ndx) {
        this.m_path.push(new gluVarTypeUtil.VarTypeComponent(type, ndx));
        if (!this.isValid()) {
            throw new Error;
        }
        return this;
    };

    gluVarTypeUtil.SubTypeAccess.prototype.member = function(ndx) {
        return this.helper(gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER, ndx);
    };
    gluVarTypeUtil.SubTypeAccess.prototype.element = function(ndx) {
        return this.helper(gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT, ndx);
    };
    gluVarTypeUtil.SubTypeAccess.prototype.column = function(ndx) {
        return this.helper(gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN, ndx);
    };
    gluVarTypeUtil.SubTypeAccess.prototype.component = function(ndx) {
        return this.helper(gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT, ndx);
    };
    gluVarTypeUtil.SubTypeAccess.prototype.parent = function() {
        if (!this.m_path.length) {
            throw new Error;
        }
        this.m_path.pop();
        return this;
    };

    gluVarTypeUtil.SubTypeAccess.prototype.isValid = function() {
        return gluVarTypeUtil.isValidTypePath(this.m_type, this.m_path);
    };
    gluVarTypeUtil.SubTypeAccess.prototype.getType = function() {
        return gluVarTypeUtil.getVarType(this.m_type, this.m_path);
    };
    gluVarTypeUtil.SubTypeAccess.prototype.getPath = function() {
        return this.m_path;
    };
    gluVarTypeUtil.SubTypeAccess.prototype.empty = function() {
        return !this.m_path.length;
    };
    gluVarTypeUtil.SubTypeAccess.prototype.is = function(other) {
        return (
            gluVarTypeUtil.array_op_equivalent(this.m_path, other.m_path) &&
            this.m_type.is(other.m_type)
        );
    };
    gluVarTypeUtil.SubTypeAccess.prototype.isnt = function(other) {
        return (
            !gluVarTypeUtil.array_op_equivalent(this.m_path, other.m_path) ||
            this.m_type.isnt(other.m_type)
        );
    };

    /**
     * Subtype iterator parent class.
     * basic usage for all child classes:
     *     for (var i = new gluVarTypeUtil.BasicTypeIterator(type) ; !i.end() ; i.next()) {
     *         var j = i.getType();
     *     }
     * @constructor
     */
    gluVarTypeUtil.SubTypeIterator = function(type) {

        /** @private */
        this.m_type = null; // const VarType*
        /** @private */
        this.m_path = []; // TypeComponentVector

        if (type) {
            this.m_type = type;
            this.findNext();
        }

    };

    gluVarTypeUtil.SubTypeIterator.prototype.isExpanded = function(type) {
        throw new Error('This function must be overriden in child class');
    };

    /** removeTraversed
     * @private
     */
    gluVarTypeUtil.SubTypeIterator.prototype.removeTraversed = function() {

        while (this.m_path.length) {
            var curComp = this.m_path[this.m_path.length - 1]; // gluVarTypeUtil.VarTypeComponent&
            var parentType = gluVarTypeUtil.getVarType(this.m_type, this.m_path, 0, this.m_path.length - 1); // VarType

            if (curComp.type == gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN) {
                if (!gluShaderUtil.isDataTypeMatrix(parentType.getBasicType())) {
                    throw new Error('Isn\'t a matrix.');
                }
                if (curComp.index + 1 < gluShaderUtil.getDataTypeMatrixNumColumns(parentType.getBasicType())) {
                    break;
                }

            } else if (curComp.type == gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT) {
                if (!gluShaderUtil.isDataTypeVector(parentType.getBasicType())) {
                    throw new Error('Isn\'t a vector.');
                }
                if (curComp.index + 1 < gluShaderUtil.getDataTypeScalarSize(parentType.getBasicType())) {
                    break;
                }

            } else if (curComp.type == gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT) {
                if (!parentType.isArrayType()) {
                    throw new Error('Isn\'t an array.');
                }
                if (curComp.index + 1 < parentType.getArraySize()) {
                    break;
                }

            } else if (curComp.type == gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER) {
                if (!parentType.isStructType()) {
                    throw new Error('Isn\'t a struct.');
                }
                if (curComp.index + 1 < parentType.getStruct().getNumMembers()) {
                    break;
                }

            }

            this.m_path.pop();
        }
    };
    gluVarTypeUtil.SubTypeIterator.prototype.findNext = function() {

        if (this.m_path.length > 0) {
            // Increment child counter in current level.
            var curComp = this.m_path[this.m_path.length - 1]; // gluVarTypeUtil.VarTypeComponent&
            curComp.index += 1;
        }

        for (;;) {

            var curType = gluVarTypeUtil.getVarType(this.m_type, this.m_path); // VarType

            if (this.isExpanded(curType))
                break;

            // Recurse into child type.
            if (curType.isBasicType()) {
                var basicType = curType.getBasicType(); // DataType

                if (gluShaderUtil.isDataTypeMatrix(basicType)) {
                    this.m_path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN, 0));

                } else if (gluShaderUtil.isDataTypeVector(basicType)) {
                    this.m_path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT, 0));

                } else {
                    throw new Error('Cant expand scalars - isExpanded() is buggy.');
                }

            } else if (curType.isArrayType()) {
                this.m_path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT, 0));

            } else if (curType.isStructType()) {
                this.m_path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER, 0));

            } else {
                throw new Error();
            }
        }

    };
    gluVarTypeUtil.SubTypeIterator.prototype.end = function() {
        return (this.m_type == null);
    };
    /** next
     * equivelant to operator++(), doesnt return.
     */
    gluVarTypeUtil.SubTypeIterator.prototype.next = function() {
        if (this.m_path.length > 0) {
            // Remove traversed nodes.
            this.removeTraversed();

            if (this.m_path.length > 0)
                this.findNext();
            else
                this.m_type = null; // Unset type to signal end.
        } else {
            if (!this.isExpanded(gluVarTypeUtil.getVarType(this.m_type, this.m_path))) {
                throw new Error('First type was already expanded.');
            }
            this.m_type = null;
        }
    };
    gluVarTypeUtil.SubTypeIterator.prototype.getType = function() {
        return gluVarTypeUtil.getVarType(this.m_type, this.m_path);
    };
    gluVarTypeUtil.SubTypeIterator.prototype.getPath = function() {
        return this.m_path;
    };

    gluVarTypeUtil.SubTypeIterator.prototype.toString = function() {
        var x = new gluVarTypeUtil.TypeAccessFormat(this.m_type, this.m_path);
        return x.toString();
    };

    /** gluVarTypeUtil.BasicTypeIterator
     * @param {gluVarType.VarType} type
     * @constructor
     * @extends {gluVarTypeUtil.SubTypeIterator}
     */
    gluVarTypeUtil.BasicTypeIterator = function(type) {
        gluVarTypeUtil.SubTypeIterator.call(this, type);
    };
    gluVarTypeUtil.BasicTypeIterator.prototype = Object.create(gluVarTypeUtil.SubTypeIterator.prototype);
    gluVarTypeUtil.BasicTypeIterator.prototype.constructor = gluVarTypeUtil.BasicTypeIterator;

    gluVarTypeUtil.BasicTypeIterator.prototype.isExpanded = function(type) {
        return type.isBasicType();
    };

    /** gluVarTypeUtil.VectorTypeIterator
     * @param {gluVarType.VarType} type
     * @constructor
     * @extends {gluVarTypeUtil.SubTypeIterator}
     */
    gluVarTypeUtil.VectorTypeIterator = function(type) {
        gluVarTypeUtil.SubTypeIterator.call(this, type);
    };
    gluVarTypeUtil.VectorTypeIterator.prototype = Object.create(gluVarTypeUtil.SubTypeIterator.prototype);
    gluVarTypeUtil.VectorTypeIterator.prototype.constructor = gluVarTypeUtil.VectorTypeIterator;

    gluVarTypeUtil.VectorTypeIterator.prototype.isExpanded = function(type) {
        return type.isBasicType() && gluShaderUtil.isDataTypeScalarOrVector(type.getBasicType());
    };

    /** gluVarTypeUtil.ScalarTypeIterator
     * @param {gluVarType.VarType} type
     * @constructor
     * @extends {gluVarTypeUtil.SubTypeIterator}
     */
     gluVarTypeUtil.ScalarTypeIterator = function(type) {
        gluVarTypeUtil.SubTypeIterator.call(this, type);
    };
    gluVarTypeUtil.ScalarTypeIterator.prototype = Object.create(gluVarTypeUtil.SubTypeIterator.prototype);
    gluVarTypeUtil.ScalarTypeIterator.prototype.constructor = gluVarTypeUtil.ScalarTypeIterator;

    gluVarTypeUtil.ScalarTypeIterator.prototype.isExpanded = function(type) {
        return type.isBasicType() && gluShaderUtil.isDataTypeScalar(type.getBasicType());
    };

    gluVarTypeUtil.inBounds = (function(x, a, b) { return a <= x && x < b; });

    /** gluVarTypeUtil.isValidTypePath
     * @param {gluVarType.VarType} type
     * @param {Array<gluVarTypeUtil.VarTypeComponent>} array
     * @param {number=} begin
     * @param {number=} end
     * @return {boolean}
     */
    gluVarTypeUtil.isValidTypePath = function(type, array, begin, end) {

        if (typeof(begin) == 'undefined') {begin = 0;}
        if (typeof(end) == 'undefined') {begin = array.length;}

        var curType = type; // const VarType*
        var pathIter = begin; // Iterator

        // Process struct member and array element parts of path.
        while (pathIter != end) {
            var element = array[pathIter];

            if (element.type == gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER) {

                if (!curType.isStructType() || !gluVarTypeUtil.inBounds(element.index, 0, curType.getStruct().getNumMembers())) {
                    return false;
                }

                curType = curType.getStruct().getMember(element.index).getType();

            } else if (element.type == gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT) {
                if (
                    !curType.isArrayType() ||
                    (
                        curType.getArraySize() != gluVarType.VarType.UNSIZED_ARRAY &&
                        !gluVarTypeUtil.inBounds(element.index, 0, curType.getArraySize())
                    )
                ) {
                    return false;
                }

                curType = curType.getElementType();
            } else {
                break;
            }

            ++pathIter;
        }

        if (pathIter != end) {
            if (!(
                array[pathIter].type == gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN ||
                array[pathIter].type == gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT
            )) {
                throw new Error('Not a matrix or a vector');
            }

            // Current type should be basic type.
            if (!curType.isBasicType()) {
                return false;
            }

            var basicType = curType.getBasicType(); // DataType

            if (array[pathIter].type == gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN) {
                if (!gluShaderUtil.isDataTypeMatrix(basicType)) {
                    return false;
                }

                basicType = gluShaderUtil.getDataTypeFloatVec(gluShaderUtil.getDataTypeMatrixNumRows(basicType));
                ++pathIter;
            }

            if (pathIter != end && array[pathIter].type == gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT) {
                if (!gluShaderUtil.isDataTypeVector(basicType))
                    return false;

                basicType = gluShaderUtil.getDataTypeScalarType(basicType);
                ++pathIter;
            }
        }

        return pathIter == end;
    };

    /** gluVarTypeUtil.getVarType
     * @param {gluVarType.VarType} type
     * @param {Array<gluVarTypeUtil.VarTypeComponent>} array
     * @param {number=} start
     * @param {number=} end
     * @return {gluVarType.VarType}
     */
    gluVarTypeUtil.getVarType = function(type, array, start, end) {

        if (typeof(start) == 'undefined') start = 0;
        if (typeof(end) == 'undefined') end = array.length;

        if (!gluVarTypeUtil.isValidTypePath(type, array, start, end)) {
            throw new Error('Type is invalid');
        }

        var curType = type; // const VarType*
        var element = null; // Iterator
        var pathIter = 0;

        // Process struct member and array element parts of path.
        for (pathIter = start; pathIter != end; ++pathIter) {
            element = array[pathIter];

            if (element.type == gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER) {
                curType = curType.getStruct().getMember(element.index).getType();

            } else if (element.type == gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT) {
                curType = curType.getElementType();

            } else {
                break;

            }
        }

        if (pathIter != end) {

            var basicType = curType.getBasicType(); // DataType
            var precision = curType.getPrecision(); // Precision

            if (element.type == gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN) {
                basicType = gluShaderUtil.getDataTypeFloatVec(gluShaderUtil.getDataTypeMatrixNumRows(basicType));
                element = array[++pathIter];
            }

            if (pathIter != end && element.type == gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT) {
                basicType = gluShaderUtil.getDataTypeScalarTypeAsDataType(basicType);
                element = array[++pathIter];
            }

            if (pathIter != end) {
                throw new Error();
            }
            return gluVarType.newTypeBasic(basicType, precision);
        } else {
            /* TODO: Original code created an object copy. We are returning reference to the same object */
            return curType;
        }
    };

    gluVarTypeUtil.parseVariableName = function(nameWithPath) {
        var tokenizer = new gluVarTypeUtil.VarTokenizer(nameWithPath);
        if (tokenizer.getToken() != gluVarTypeUtil.VarTokenizer.s_Token.IDENTIFIER) {
            throw new Error('Not an identifier.');
        }
        return tokenizer.getIdentifier();
    };

    // returns an array (TypeComponentVector& path)
    // params: const char*, const VarType&
    gluVarTypeUtil.parseTypePath = function(nameWithPath, type) {

        var tokenizer = new gluVarTypeUtil.VarTokenizer(nameWithPath);

        if (tokenizer.getToken() == gluVarTypeUtil.VarTokenizer.s_Token.IDENTIFIER) {
            tokenizer.advance();
        }

        var path = [];

        while (tokenizer.getToken() != gluVarTypeUtil.VarTokenizer.s_Token.END) {

            var curType = gluVarTypeUtil.getVarType(type, path);

            if (tokenizer.getToken() == gluVarTypeUtil.VarTokenizer.s_Token.PERIOD) {

                tokenizer.advance();
                if (tokenizer.getToken() != gluVarTypeUtil.VarTokenizer.s_Token.IDENTIFIER) {
                    throw new Error();
                }
                if (!curType.isStructType()) {
                    throw new Error('Invalid field selector');
                }

                // Find member.
                var memberName = tokenizer.getIdentifier();
                var ndx = 0;
                for (; ndx < curType.getStruct().getSize(); ++ndx) {

                    if (memberName == curType.getStruct().getMember(ndx).getName()) {
                        break;
                    }

                }
                if (ndx >= curType.getStruct().getSize()) {
                    throw new Error('Member not found in type: ' + memberName);
                }

                path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER, ndx));
                tokenizer.advance();

            } else if (tokenizer.getToken() == gluVarTypeUtil.VarTokenizer.s_Token.LEFT_BRACKET) {

                tokenizer.advance();
                if (tokenizer.getToken() != gluVarTypeUtil.VarTokenizer.s_Token.NUMBER) {
                    throw new Error();
                }

                var ndx = tokenizer.getNumber();

                if (curType.isArrayType()) {
                    if (!gluVarTypeUtil.inBounds(ndx, 0, curType.getArraySize())) throw new Error;
                    path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT, ndx));

                } else if (curType.isBasicType() && gluShaderUtil.isDataTypeMatrix(curType.getBasicType())) {
                    if (!gluVarTypeUtil.inBounds(ndx, 0, gluShaderUtil.getDataTypeMatrixNumColumns(curType.getBasicType()))) throw new Error;
                    path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN, ndx));

                } else if (curType.isBasicType() && gluShaderUtil.isDataTypeVector(curType.getBasicType())) {
                    if (!gluVarTypeUtil.inBounds(ndx, 0, gluShaderUtil.getDataTypeScalarSize(curType.getBasicType()))) throw new Error;
                    path.push(new gluVarTypeUtil.VarTypeComponent(gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT, ndx));

                } else {
                    //TCU_FAIL
                    throw new Error('Invalid subscript');
                }

                tokenizer.advance();
                if (tokenizer.getToken() != gluVarTypeUtil.VarTokenizer.s_Token.RIGHT_BRACKET) {
                    throw new Error('Expected token RIGHT_BRACKET');
                }
                tokenizer.advance();

            } else {
                // TCU_FAIL
                throw new Error('Unexpected token');
            }
        }

        return path;

    };

});
