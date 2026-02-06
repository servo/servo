
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "glsl_optimizer.h"

static glslopt_ctx* gContext = 0;

static int printhelp(const char* msg)
{
	if (msg) printf("%s\n\n\n", msg);
	printf("Usage: glslopt <-f|-v> <input shader> [<output shader>]\n");
	printf("\t-f : fragment shader (default)\n");
	printf("\t-v : vertex shader\n");
	printf("\t-1 : target OpenGL (default)\n");
	printf("\t-2 : target OpenGL ES 2.0\n");
	printf("\t-3 : target OpenGL ES 3.0\n");
	printf("\n\tIf no output specified, output is to [input].out.\n");
	return 1;
}

static bool init(glslopt_target target)
{
	gContext = glslopt_initialize(target);
	if( !gContext )
		return false;
	return true;
}

static void term()
{
	glslopt_cleanup(gContext);
}

static char* loadFile(const char* filename)
{
	FILE* file = fopen(filename, "rt");
	if( !file )
	{
		printf("Failed to open %s for reading\n", filename);
		return 0;
	}

	fseek(file, 0, SEEK_END);
	const int size = ftell(file);
	fseek(file, 0, SEEK_SET);

	char* result = new char[size+1];
	const int count = (int)fread(result, 1, size, file);
	result[count] = 0;

	fclose(file);
	return result;
}

static bool saveFile(const char* filename, const char* data)
{
	int size = (int)strlen(data);

	FILE* file = fopen(filename, "wt");
	if( !file )
	{
		printf( "Failed to open %s for writing\n", filename);
		return false;
	}

	if( 1 != fwrite(data,size,1,file) )
	{
		printf( "Failed to write to %s\n", filename);
		fclose(file);
		return false;
	}

	fclose(file);
	return true;
}

static bool compileShader(const char* dstfilename, const char* srcfilename, bool vertexShader)
{
	const char* originalShader = loadFile(srcfilename);
	if( !originalShader )
		return false;

	const glslopt_shader_type type = vertexShader ? kGlslOptShaderVertex : kGlslOptShaderFragment;

	glslopt_shader* shader = glslopt_optimize(gContext, type, originalShader, 0);
	if( !glslopt_get_status(shader) )
	{
		printf( "Failed to compile %s:\n\n%s\n", srcfilename, glslopt_get_log(shader));
		return false;
	}

	const char* optimizedShader = glslopt_get_output(shader);

	if( !saveFile(dstfilename, optimizedShader) )
		return false;

	delete[] originalShader;
	return true;
}

int main(int argc, char* argv[])
{
	if( argc < 3 )
		return printhelp(NULL);

	bool vertexShader = false, freename = false;
	glslopt_target languageTarget = kGlslTargetOpenGL;
	const char* source = 0;
	char* dest = 0;

	for( int i=1; i < argc; i++ )
	{
		if( argv[i][0] == '-' )
		{
			if( 0 == strcmp("-v", argv[i]) )
				vertexShader = true;
			else if( 0 == strcmp("-f", argv[i]) )
				vertexShader = false;
			else if( 0 == strcmp("-1", argv[i]) )
				languageTarget = kGlslTargetOpenGL;
			else if( 0 == strcmp("-2", argv[i]) )
				languageTarget = kGlslTargetOpenGLES20;
			else if( 0 == strcmp("-3", argv[i]) )
				languageTarget = kGlslTargetOpenGLES30;
		}
		else
		{
			if( source == 0 )
				source = argv[i];
			else if( dest == 0 )
				dest = argv[i];
		}
	}

	if( !source )
		return printhelp("Must give a source");

	if( !init(languageTarget) )
	{
		printf("Failed to initialize glslopt!\n");
		return 1;
	}

	if ( !dest ) {
		dest = (char *) calloc(strlen(source)+5, sizeof(char));
		snprintf(dest, strlen(source)+5, "%s.out", source);
		freename = true;
	}

	int result = 0;
	if( !compileShader(dest, source, vertexShader) )
		result = 1;

	if( freename ) free(dest);

	term();
	return result;
}
