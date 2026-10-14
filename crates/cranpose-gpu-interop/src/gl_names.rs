// Written by scripts/gl-names.py from Emscripten 4.0.20.

use std::ffi::c_void;

// Taking a symbol's address makes Emscripten's dynamic linker resolve it
// against the main module's exports; nothing calls these declarations.
unsafe extern "C" {
    fn glActiveTexture();
    fn glAttachShader();
    fn glBeginQuery();
    fn glBeginQueryEXT();
    fn glBeginTransformFeedback();
    fn glBindAttribLocation();
    fn glBindBuffer();
    fn glBindBufferBase();
    fn glBindBufferRange();
    fn glBindFramebuffer();
    fn glBindRenderbuffer();
    fn glBindSampler();
    fn glBindTexture();
    fn glBindTransformFeedback();
    fn glBindVertexArray();
    fn glBindVertexArrayOES();
    fn glBlendColor();
    fn glBlendEquation();
    fn glBlendEquationSeparate();
    fn glBlendFunc();
    fn glBlendFuncSeparate();
    fn glBlitFramebuffer();
    fn glBufferData();
    fn glBufferSubData();
    fn glCheckFramebufferStatus();
    fn glClear();
    fn glClearBufferfi();
    fn glClearBufferfv();
    fn glClearBufferiv();
    fn glClearBufferuiv();
    fn glClearColor();
    fn glClearDepthf();
    fn glClearStencil();
    fn glClientWaitSync();
    fn glClipControlEXT();
    fn glColorMask();
    fn glCompileShader();
    fn glCompressedTexImage2D();
    fn glCompressedTexImage3D();
    fn glCompressedTexSubImage2D();
    fn glCompressedTexSubImage3D();
    fn glCopyBufferSubData();
    fn glCopyTexImage2D();
    fn glCopyTexSubImage2D();
    fn glCopyTexSubImage3D();
    fn glCreateProgram();
    fn glCreateShader();
    fn glCullFace();
    fn glDeleteBuffers();
    fn glDeleteFramebuffers();
    fn glDeleteProgram();
    fn glDeleteQueries();
    fn glDeleteQueriesEXT();
    fn glDeleteRenderbuffers();
    fn glDeleteSamplers();
    fn glDeleteShader();
    fn glDeleteSync();
    fn glDeleteTextures();
    fn glDeleteTransformFeedbacks();
    fn glDeleteVertexArrays();
    fn glDeleteVertexArraysOES();
    fn glDepthFunc();
    fn glDepthMask();
    fn glDepthRangef();
    fn glDetachShader();
    fn glDisable();
    fn glDisableVertexAttribArray();
    fn glDrawArrays();
    fn glDrawArraysInstanced();
    fn glDrawArraysInstancedANGLE();
    fn glDrawArraysInstancedARB();
    fn glDrawArraysInstancedEXT();
    fn glDrawArraysInstancedNV();
    fn glDrawBuffers();
    fn glDrawBuffersEXT();
    fn glDrawBuffersWEBGL();
    fn glDrawElements();
    fn glDrawElementsInstanced();
    fn glDrawElementsInstancedANGLE();
    fn glDrawElementsInstancedARB();
    fn glDrawElementsInstancedEXT();
    fn glDrawElementsInstancedNV();
    fn glDrawRangeElements();
    fn glEnable();
    fn glEnableVertexAttribArray();
    fn glEndQuery();
    fn glEndQueryEXT();
    fn glEndTransformFeedback();
    fn glFenceSync();
    fn glFinish();
    fn glFlush();
    fn glFramebufferRenderbuffer();
    fn glFramebufferTexture2D();
    fn glFramebufferTextureLayer();
    fn glFrontFace();
    fn glGenBuffers();
    fn glGenFramebuffers();
    fn glGenQueries();
    fn glGenQueriesEXT();
    fn glGenRenderbuffers();
    fn glGenSamplers();
    fn glGenTextures();
    fn glGenTransformFeedbacks();
    fn glGenVertexArrays();
    fn glGenVertexArraysOES();
    fn glGenerateMipmap();
    fn glGetActiveAttrib();
    fn glGetActiveUniform();
    fn glGetActiveUniformBlockName();
    fn glGetActiveUniformBlockiv();
    fn glGetActiveUniformsiv();
    fn glGetAttachedShaders();
    fn glGetAttribLocation();
    fn glGetBooleanv();
    fn glGetBufferParameteri64v();
    fn glGetBufferParameteriv();
    fn glGetError();
    fn glGetFloatv();
    fn glGetFragDataLocation();
    fn glGetFramebufferAttachmentParameteriv();
    fn glGetInteger64i_v();
    fn glGetInteger64v();
    fn glGetIntegeri_v();
    fn glGetIntegerv();
    fn glGetInternalformativ();
    fn glGetProgramBinary();
    fn glGetProgramInfoLog();
    fn glGetProgramiv();
    fn glGetQueryObjecti64vEXT();
    fn glGetQueryObjectivEXT();
    fn glGetQueryObjectui64vEXT();
    fn glGetQueryObjectuiv();
    fn glGetQueryObjectuivEXT();
    fn glGetQueryiv();
    fn glGetQueryivEXT();
    fn glGetRenderbufferParameteriv();
    fn glGetSamplerParameterfv();
    fn glGetSamplerParameteriv();
    fn glGetShaderInfoLog();
    fn glGetShaderPrecisionFormat();
    fn glGetShaderSource();
    fn glGetShaderiv();
    fn glGetString();
    fn glGetStringi();
    fn glGetSynciv();
    fn glGetTexParameterfv();
    fn glGetTexParameteriv();
    fn glGetTransformFeedbackVarying();
    fn glGetUniformBlockIndex();
    fn glGetUniformIndices();
    fn glGetUniformLocation();
    fn glGetUniformfv();
    fn glGetUniformiv();
    fn glGetUniformuiv();
    fn glGetVertexAttribIiv();
    fn glGetVertexAttribIuiv();
    fn glGetVertexAttribPointerv();
    fn glGetVertexAttribfv();
    fn glGetVertexAttribiv();
    fn glHint();
    fn glInvalidateFramebuffer();
    fn glInvalidateSubFramebuffer();
    fn glIsBuffer();
    fn glIsEnabled();
    fn glIsFramebuffer();
    fn glIsProgram();
    fn glIsQuery();
    fn glIsQueryEXT();
    fn glIsRenderbuffer();
    fn glIsSampler();
    fn glIsShader();
    fn glIsSync();
    fn glIsTexture();
    fn glIsTransformFeedback();
    fn glIsVertexArray();
    fn glIsVertexArrayOES();
    fn glLineWidth();
    fn glLinkProgram();
    fn glPauseTransformFeedback();
    fn glPixelStorei();
    fn glPolygonModeWEBGL();
    fn glPolygonOffset();
    fn glPolygonOffsetClampEXT();
    fn glProgramBinary();
    fn glProgramParameteri();
    fn glQueryCounterEXT();
    fn glReadBuffer();
    fn glReadPixels();
    fn glReleaseShaderCompiler();
    fn glRenderbufferStorage();
    fn glRenderbufferStorageMultisample();
    fn glResumeTransformFeedback();
    fn glSampleCoverage();
    fn glSamplerParameterf();
    fn glSamplerParameterfv();
    fn glSamplerParameteri();
    fn glSamplerParameteriv();
    fn glScissor();
    fn glShaderBinary();
    fn glShaderSource();
    fn glStencilFunc();
    fn glStencilFuncSeparate();
    fn glStencilMask();
    fn glStencilMaskSeparate();
    fn glStencilOp();
    fn glStencilOpSeparate();
    fn glTexImage2D();
    fn glTexImage3D();
    fn glTexParameterf();
    fn glTexParameterfv();
    fn glTexParameteri();
    fn glTexParameteriv();
    fn glTexStorage2D();
    fn glTexStorage3D();
    fn glTexSubImage2D();
    fn glTexSubImage3D();
    fn glTransformFeedbackVaryings();
    fn glUniform1f();
    fn glUniform1fv();
    fn glUniform1i();
    fn glUniform1iv();
    fn glUniform1ui();
    fn glUniform1uiv();
    fn glUniform2f();
    fn glUniform2fv();
    fn glUniform2i();
    fn glUniform2iv();
    fn glUniform2ui();
    fn glUniform2uiv();
    fn glUniform3f();
    fn glUniform3fv();
    fn glUniform3i();
    fn glUniform3iv();
    fn glUniform3ui();
    fn glUniform3uiv();
    fn glUniform4f();
    fn glUniform4fv();
    fn glUniform4i();
    fn glUniform4iv();
    fn glUniform4ui();
    fn glUniform4uiv();
    fn glUniformBlockBinding();
    fn glUniformMatrix2fv();
    fn glUniformMatrix2x3fv();
    fn glUniformMatrix2x4fv();
    fn glUniformMatrix3fv();
    fn glUniformMatrix3x2fv();
    fn glUniformMatrix3x4fv();
    fn glUniformMatrix4fv();
    fn glUniformMatrix4x2fv();
    fn glUniformMatrix4x3fv();
    fn glUseProgram();
    fn glValidateProgram();
    fn glVertexAttrib1f();
    fn glVertexAttrib1fv();
    fn glVertexAttrib2f();
    fn glVertexAttrib2fv();
    fn glVertexAttrib3f();
    fn glVertexAttrib3fv();
    fn glVertexAttrib4f();
    fn glVertexAttrib4fv();
    fn glVertexAttribDivisor();
    fn glVertexAttribDivisorANGLE();
    fn glVertexAttribDivisorARB();
    fn glVertexAttribDivisorEXT();
    fn glVertexAttribDivisorNV();
    fn glVertexAttribI4i();
    fn glVertexAttribI4iv();
    fn glVertexAttribI4ui();
    fn glVertexAttribI4uiv();
    fn glVertexAttribIPointer();
    fn glVertexAttribPointer();
    fn glViewport();
    fn glWaitSync();
}

/// The address of the WebGL entry point Emscripten names `name`, or null.
pub(crate) fn gl_proc_address(name: &str) -> *const c_void {
    match name {
        "glActiveTexture" => glActiveTexture as *const c_void,
        "glAttachShader" => glAttachShader as *const c_void,
        "glBindAttribLocation" => glBindAttribLocation as *const c_void,
        "glBindBuffer" => glBindBuffer as *const c_void,
        "glBindFramebuffer" => glBindFramebuffer as *const c_void,
        "glBindRenderbuffer" => glBindRenderbuffer as *const c_void,
        "glBindTexture" => glBindTexture as *const c_void,
        "glBlendColor" => glBlendColor as *const c_void,
        "glBlendEquation" => glBlendEquation as *const c_void,
        "glBlendEquationSeparate" => glBlendEquationSeparate as *const c_void,
        "glBlendFunc" => glBlendFunc as *const c_void,
        "glBlendFuncSeparate" => glBlendFuncSeparate as *const c_void,
        "glBufferData" => glBufferData as *const c_void,
        "glBufferSubData" => glBufferSubData as *const c_void,
        "glCheckFramebufferStatus" => glCheckFramebufferStatus as *const c_void,
        "glClear" => glClear as *const c_void,
        "glClearColor" => glClearColor as *const c_void,
        "glClearDepthf" => glClearDepthf as *const c_void,
        "glClearStencil" => glClearStencil as *const c_void,
        "glColorMask" => glColorMask as *const c_void,
        "glCompileShader" => glCompileShader as *const c_void,
        "glCompressedTexImage2D" => glCompressedTexImage2D as *const c_void,
        "glCompressedTexSubImage2D" => glCompressedTexSubImage2D as *const c_void,
        "glCopyTexImage2D" => glCopyTexImage2D as *const c_void,
        "glCopyTexSubImage2D" => glCopyTexSubImage2D as *const c_void,
        "glCreateProgram" => glCreateProgram as *const c_void,
        "glCreateShader" => glCreateShader as *const c_void,
        "glCullFace" => glCullFace as *const c_void,
        "glDeleteBuffers" => glDeleteBuffers as *const c_void,
        "glDeleteFramebuffers" => glDeleteFramebuffers as *const c_void,
        "glDeleteProgram" => glDeleteProgram as *const c_void,
        "glDeleteRenderbuffers" => glDeleteRenderbuffers as *const c_void,
        "glDeleteShader" => glDeleteShader as *const c_void,
        "glDeleteTextures" => glDeleteTextures as *const c_void,
        "glDepthFunc" => glDepthFunc as *const c_void,
        "glDepthMask" => glDepthMask as *const c_void,
        "glDepthRangef" => glDepthRangef as *const c_void,
        "glDetachShader" => glDetachShader as *const c_void,
        "glDisable" => glDisable as *const c_void,
        "glDisableVertexAttribArray" => glDisableVertexAttribArray as *const c_void,
        "glDrawArrays" => glDrawArrays as *const c_void,
        "glDrawElements" => glDrawElements as *const c_void,
        "glEnable" => glEnable as *const c_void,
        "glEnableVertexAttribArray" => glEnableVertexAttribArray as *const c_void,
        "glFinish" => glFinish as *const c_void,
        "glFlush" => glFlush as *const c_void,
        "glFramebufferRenderbuffer" => glFramebufferRenderbuffer as *const c_void,
        "glFramebufferTexture2D" => glFramebufferTexture2D as *const c_void,
        "glFrontFace" => glFrontFace as *const c_void,
        "glGenBuffers" => glGenBuffers as *const c_void,
        "glGenerateMipmap" => glGenerateMipmap as *const c_void,
        "glGenFramebuffers" => glGenFramebuffers as *const c_void,
        "glGenRenderbuffers" => glGenRenderbuffers as *const c_void,
        "glGenTextures" => glGenTextures as *const c_void,
        "glGetActiveAttrib" => glGetActiveAttrib as *const c_void,
        "glGetActiveUniform" => glGetActiveUniform as *const c_void,
        "glGetAttachedShaders" => glGetAttachedShaders as *const c_void,
        "glGetAttribLocation" => glGetAttribLocation as *const c_void,
        "glGetBooleanv" => glGetBooleanv as *const c_void,
        "glGetBufferParameteriv" => glGetBufferParameteriv as *const c_void,
        "glGetError" => glGetError as *const c_void,
        "glGetFloatv" => glGetFloatv as *const c_void,
        "glGetFramebufferAttachmentParameteriv" => {
            glGetFramebufferAttachmentParameteriv as *const c_void
        }
        "glGetIntegerv" => glGetIntegerv as *const c_void,
        "glGetProgramiv" => glGetProgramiv as *const c_void,
        "glGetProgramInfoLog" => glGetProgramInfoLog as *const c_void,
        "glGetRenderbufferParameteriv" => glGetRenderbufferParameteriv as *const c_void,
        "glGetShaderiv" => glGetShaderiv as *const c_void,
        "glGetShaderInfoLog" => glGetShaderInfoLog as *const c_void,
        "glGetShaderPrecisionFormat" => glGetShaderPrecisionFormat as *const c_void,
        "glGetShaderSource" => glGetShaderSource as *const c_void,
        "glGetString" => glGetString as *const c_void,
        "glGetTexParameterfv" => glGetTexParameterfv as *const c_void,
        "glGetTexParameteriv" => glGetTexParameteriv as *const c_void,
        "glGetUniformfv" => glGetUniformfv as *const c_void,
        "glGetUniformiv" => glGetUniformiv as *const c_void,
        "glGetUniformLocation" => glGetUniformLocation as *const c_void,
        "glGetVertexAttribfv" => glGetVertexAttribfv as *const c_void,
        "glGetVertexAttribiv" => glGetVertexAttribiv as *const c_void,
        "glGetVertexAttribPointerv" => glGetVertexAttribPointerv as *const c_void,
        "glHint" => glHint as *const c_void,
        "glIsBuffer" => glIsBuffer as *const c_void,
        "glIsEnabled" => glIsEnabled as *const c_void,
        "glIsFramebuffer" => glIsFramebuffer as *const c_void,
        "glIsProgram" => glIsProgram as *const c_void,
        "glIsRenderbuffer" => glIsRenderbuffer as *const c_void,
        "glIsShader" => glIsShader as *const c_void,
        "glIsTexture" => glIsTexture as *const c_void,
        "glLineWidth" => glLineWidth as *const c_void,
        "glLinkProgram" => glLinkProgram as *const c_void,
        "glPixelStorei" => glPixelStorei as *const c_void,
        "glPolygonOffset" => glPolygonOffset as *const c_void,
        "glReadPixels" => glReadPixels as *const c_void,
        "glReleaseShaderCompiler" => glReleaseShaderCompiler as *const c_void,
        "glRenderbufferStorage" => glRenderbufferStorage as *const c_void,
        "glSampleCoverage" => glSampleCoverage as *const c_void,
        "glScissor" => glScissor as *const c_void,
        "glShaderBinary" => glShaderBinary as *const c_void,
        "glShaderSource" => glShaderSource as *const c_void,
        "glStencilFunc" => glStencilFunc as *const c_void,
        "glStencilFuncSeparate" => glStencilFuncSeparate as *const c_void,
        "glStencilMask" => glStencilMask as *const c_void,
        "glStencilMaskSeparate" => glStencilMaskSeparate as *const c_void,
        "glStencilOp" => glStencilOp as *const c_void,
        "glStencilOpSeparate" => glStencilOpSeparate as *const c_void,
        "glTexImage2D" => glTexImage2D as *const c_void,
        "glTexParameterf" => glTexParameterf as *const c_void,
        "glTexParameterfv" => glTexParameterfv as *const c_void,
        "glTexParameteri" => glTexParameteri as *const c_void,
        "glTexParameteriv" => glTexParameteriv as *const c_void,
        "glTexSubImage2D" => glTexSubImage2D as *const c_void,
        "glUniform1f" => glUniform1f as *const c_void,
        "glUniform1fv" => glUniform1fv as *const c_void,
        "glUniform1i" => glUniform1i as *const c_void,
        "glUniform1iv" => glUniform1iv as *const c_void,
        "glUniform2f" => glUniform2f as *const c_void,
        "glUniform2fv" => glUniform2fv as *const c_void,
        "glUniform2i" => glUniform2i as *const c_void,
        "glUniform2iv" => glUniform2iv as *const c_void,
        "glUniform3f" => glUniform3f as *const c_void,
        "glUniform3fv" => glUniform3fv as *const c_void,
        "glUniform3i" => glUniform3i as *const c_void,
        "glUniform3iv" => glUniform3iv as *const c_void,
        "glUniform4f" => glUniform4f as *const c_void,
        "glUniform4fv" => glUniform4fv as *const c_void,
        "glUniform4i" => glUniform4i as *const c_void,
        "glUniform4iv" => glUniform4iv as *const c_void,
        "glUniformMatrix2fv" => glUniformMatrix2fv as *const c_void,
        "glUniformMatrix3fv" => glUniformMatrix3fv as *const c_void,
        "glUniformMatrix4fv" => glUniformMatrix4fv as *const c_void,
        "glUseProgram" => glUseProgram as *const c_void,
        "glValidateProgram" => glValidateProgram as *const c_void,
        "glVertexAttrib1f" => glVertexAttrib1f as *const c_void,
        "glVertexAttrib1fv" => glVertexAttrib1fv as *const c_void,
        "glVertexAttrib2f" => glVertexAttrib2f as *const c_void,
        "glVertexAttrib2fv" => glVertexAttrib2fv as *const c_void,
        "glVertexAttrib3f" => glVertexAttrib3f as *const c_void,
        "glVertexAttrib3fv" => glVertexAttrib3fv as *const c_void,
        "glVertexAttrib4f" => glVertexAttrib4f as *const c_void,
        "glVertexAttrib4fv" => glVertexAttrib4fv as *const c_void,
        "glVertexAttribPointer" => glVertexAttribPointer as *const c_void,
        "glViewport" => glViewport as *const c_void,
        "glGenQueriesEXT" => glGenQueriesEXT as *const c_void,
        "glDeleteQueriesEXT" => glDeleteQueriesEXT as *const c_void,
        "glIsQueryEXT" => glIsQueryEXT as *const c_void,
        "glBeginQueryEXT" => glBeginQueryEXT as *const c_void,
        "glEndQueryEXT" => glEndQueryEXT as *const c_void,
        "glQueryCounterEXT" => glQueryCounterEXT as *const c_void,
        "glGetQueryivEXT" => glGetQueryivEXT as *const c_void,
        "glGetQueryObjectivEXT" => glGetQueryObjectivEXT as *const c_void,
        "glGetQueryObjectuivEXT" => glGetQueryObjectuivEXT as *const c_void,
        "glGetQueryObjecti64vEXT" => glGetQueryObjecti64vEXT as *const c_void,
        "glGetQueryObjectui64vEXT" => glGetQueryObjectui64vEXT as *const c_void,
        "glBindVertexArrayOES" => glBindVertexArrayOES as *const c_void,
        "glDeleteVertexArraysOES" => glDeleteVertexArraysOES as *const c_void,
        "glGenVertexArraysOES" => glGenVertexArraysOES as *const c_void,
        "glIsVertexArrayOES" => glIsVertexArrayOES as *const c_void,
        "glDrawBuffersWEBGL" => glDrawBuffersWEBGL as *const c_void,
        "glDrawArraysInstancedANGLE" => glDrawArraysInstancedANGLE as *const c_void,
        "glDrawElementsInstancedANGLE" => glDrawElementsInstancedANGLE as *const c_void,
        "glVertexAttribDivisorANGLE" => glVertexAttribDivisorANGLE as *const c_void,
        "glPolygonOffsetClampEXT" => glPolygonOffsetClampEXT as *const c_void,
        "glClipControlEXT" => glClipControlEXT as *const c_void,
        "glPolygonModeWEBGL" => glPolygonModeWEBGL as *const c_void,
        "glReadBuffer" => glReadBuffer as *const c_void,
        "glDrawRangeElements" => glDrawRangeElements as *const c_void,
        "glTexImage3D" => glTexImage3D as *const c_void,
        "glTexSubImage3D" => glTexSubImage3D as *const c_void,
        "glCopyTexSubImage3D" => glCopyTexSubImage3D as *const c_void,
        "glCompressedTexImage3D" => glCompressedTexImage3D as *const c_void,
        "glCompressedTexSubImage3D" => glCompressedTexSubImage3D as *const c_void,
        "glGenQueries" => glGenQueries as *const c_void,
        "glDeleteQueries" => glDeleteQueries as *const c_void,
        "glIsQuery" => glIsQuery as *const c_void,
        "glBeginQuery" => glBeginQuery as *const c_void,
        "glEndQuery" => glEndQuery as *const c_void,
        "glGetQueryiv" => glGetQueryiv as *const c_void,
        "glGetQueryObjectuiv" => glGetQueryObjectuiv as *const c_void,
        "glDrawBuffers" => glDrawBuffers as *const c_void,
        "glUniformMatrix2x3fv" => glUniformMatrix2x3fv as *const c_void,
        "glUniformMatrix3x2fv" => glUniformMatrix3x2fv as *const c_void,
        "glUniformMatrix2x4fv" => glUniformMatrix2x4fv as *const c_void,
        "glUniformMatrix4x2fv" => glUniformMatrix4x2fv as *const c_void,
        "glUniformMatrix3x4fv" => glUniformMatrix3x4fv as *const c_void,
        "glUniformMatrix4x3fv" => glUniformMatrix4x3fv as *const c_void,
        "glBlitFramebuffer" => glBlitFramebuffer as *const c_void,
        "glRenderbufferStorageMultisample" => glRenderbufferStorageMultisample as *const c_void,
        "glFramebufferTextureLayer" => glFramebufferTextureLayer as *const c_void,
        "glBindVertexArray" => glBindVertexArray as *const c_void,
        "glDeleteVertexArrays" => glDeleteVertexArrays as *const c_void,
        "glGenVertexArrays" => glGenVertexArrays as *const c_void,
        "glIsVertexArray" => glIsVertexArray as *const c_void,
        "glGetIntegeri_v" => glGetIntegeri_v as *const c_void,
        "glBeginTransformFeedback" => glBeginTransformFeedback as *const c_void,
        "glEndTransformFeedback" => glEndTransformFeedback as *const c_void,
        "glBindBufferRange" => glBindBufferRange as *const c_void,
        "glBindBufferBase" => glBindBufferBase as *const c_void,
        "glTransformFeedbackVaryings" => glTransformFeedbackVaryings as *const c_void,
        "glGetTransformFeedbackVarying" => glGetTransformFeedbackVarying as *const c_void,
        "glVertexAttribIPointer" => glVertexAttribIPointer as *const c_void,
        "glGetVertexAttribIiv" => glGetVertexAttribIiv as *const c_void,
        "glGetVertexAttribIuiv" => glGetVertexAttribIuiv as *const c_void,
        "glVertexAttribI4i" => glVertexAttribI4i as *const c_void,
        "glVertexAttribI4ui" => glVertexAttribI4ui as *const c_void,
        "glVertexAttribI4iv" => glVertexAttribI4iv as *const c_void,
        "glVertexAttribI4uiv" => glVertexAttribI4uiv as *const c_void,
        "glGetUniformuiv" => glGetUniformuiv as *const c_void,
        "glGetFragDataLocation" => glGetFragDataLocation as *const c_void,
        "glUniform1ui" => glUniform1ui as *const c_void,
        "glUniform2ui" => glUniform2ui as *const c_void,
        "glUniform3ui" => glUniform3ui as *const c_void,
        "glUniform4ui" => glUniform4ui as *const c_void,
        "glUniform1uiv" => glUniform1uiv as *const c_void,
        "glUniform2uiv" => glUniform2uiv as *const c_void,
        "glUniform3uiv" => glUniform3uiv as *const c_void,
        "glUniform4uiv" => glUniform4uiv as *const c_void,
        "glClearBufferiv" => glClearBufferiv as *const c_void,
        "glClearBufferuiv" => glClearBufferuiv as *const c_void,
        "glClearBufferfv" => glClearBufferfv as *const c_void,
        "glClearBufferfi" => glClearBufferfi as *const c_void,
        "glGetStringi" => glGetStringi as *const c_void,
        "glCopyBufferSubData" => glCopyBufferSubData as *const c_void,
        "glGetUniformIndices" => glGetUniformIndices as *const c_void,
        "glGetActiveUniformsiv" => glGetActiveUniformsiv as *const c_void,
        "glGetUniformBlockIndex" => glGetUniformBlockIndex as *const c_void,
        "glGetActiveUniformBlockiv" => glGetActiveUniformBlockiv as *const c_void,
        "glGetActiveUniformBlockName" => glGetActiveUniformBlockName as *const c_void,
        "glUniformBlockBinding" => glUniformBlockBinding as *const c_void,
        "glDrawArraysInstanced" => glDrawArraysInstanced as *const c_void,
        "glDrawElementsInstanced" => glDrawElementsInstanced as *const c_void,
        "glFenceSync" => glFenceSync as *const c_void,
        "glIsSync" => glIsSync as *const c_void,
        "glDeleteSync" => glDeleteSync as *const c_void,
        "glClientWaitSync" => glClientWaitSync as *const c_void,
        "glWaitSync" => glWaitSync as *const c_void,
        "glGetInteger64v" => glGetInteger64v as *const c_void,
        "glGetSynciv" => glGetSynciv as *const c_void,
        "glGetInteger64i_v" => glGetInteger64i_v as *const c_void,
        "glGetBufferParameteri64v" => glGetBufferParameteri64v as *const c_void,
        "glGenSamplers" => glGenSamplers as *const c_void,
        "glDeleteSamplers" => glDeleteSamplers as *const c_void,
        "glIsSampler" => glIsSampler as *const c_void,
        "glBindSampler" => glBindSampler as *const c_void,
        "glSamplerParameteri" => glSamplerParameteri as *const c_void,
        "glSamplerParameteriv" => glSamplerParameteriv as *const c_void,
        "glSamplerParameterf" => glSamplerParameterf as *const c_void,
        "glSamplerParameterfv" => glSamplerParameterfv as *const c_void,
        "glGetSamplerParameteriv" => glGetSamplerParameteriv as *const c_void,
        "glGetSamplerParameterfv" => glGetSamplerParameterfv as *const c_void,
        "glVertexAttribDivisor" => glVertexAttribDivisor as *const c_void,
        "glBindTransformFeedback" => glBindTransformFeedback as *const c_void,
        "glDeleteTransformFeedbacks" => glDeleteTransformFeedbacks as *const c_void,
        "glGenTransformFeedbacks" => glGenTransformFeedbacks as *const c_void,
        "glIsTransformFeedback" => glIsTransformFeedback as *const c_void,
        "glPauseTransformFeedback" => glPauseTransformFeedback as *const c_void,
        "glResumeTransformFeedback" => glResumeTransformFeedback as *const c_void,
        "glGetProgramBinary" => glGetProgramBinary as *const c_void,
        "glProgramBinary" => glProgramBinary as *const c_void,
        "glProgramParameteri" => glProgramParameteri as *const c_void,
        "glInvalidateFramebuffer" => glInvalidateFramebuffer as *const c_void,
        "glInvalidateSubFramebuffer" => glInvalidateSubFramebuffer as *const c_void,
        "glTexStorage2D" => glTexStorage2D as *const c_void,
        "glTexStorage3D" => glTexStorage3D as *const c_void,
        "glGetInternalformativ" => glGetInternalformativ as *const c_void,
        "glVertexAttribDivisorNV" => glVertexAttribDivisorNV as *const c_void,
        "glVertexAttribDivisorEXT" => glVertexAttribDivisorEXT as *const c_void,
        "glVertexAttribDivisorARB" => glVertexAttribDivisorARB as *const c_void,
        "glDrawArraysInstancedNV" => glDrawArraysInstancedNV as *const c_void,
        "glDrawArraysInstancedEXT" => glDrawArraysInstancedEXT as *const c_void,
        "glDrawArraysInstancedARB" => glDrawArraysInstancedARB as *const c_void,
        "glDrawElementsInstancedNV" => glDrawElementsInstancedNV as *const c_void,
        "glDrawElementsInstancedEXT" => glDrawElementsInstancedEXT as *const c_void,
        "glDrawElementsInstancedARB" => glDrawElementsInstancedARB as *const c_void,
        "glDrawBuffersEXT" => glDrawBuffersEXT as *const c_void,
        _ => std::ptr::null(),
    }
}
