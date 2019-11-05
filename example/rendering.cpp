shibaUniformTime = shibaTime;
shibaUniformResolutionWidth = shibaResolutionWidth;
shibaUniformResolutionHeight = shibaResolutionHeight;

// Framebuffer
glBindFramebuffer(GL_FRAMEBUFFER, fbo);
shibaCheckGlError();
glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, firstPassTextureId, 0);
shibaCheckGlError();

#ifdef SHIBA_DEVELOPMENT
static bool checked = false;
if (!checked)
{
	checked = true;
	auto status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
	shibaCheckGlError();
	if (status != GL_FRAMEBUFFER_COMPLETE)
	{
		shibaError() << "Framebuffer is incomplete!";
		return;
	}
	else
	{
		shibaLog() << "Framebuffer is complete.";
	}
}
#endif

glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
shibaCheckGlError();
glDepthMask(GL_TRUE);
shibaCheckGlError();
glEnable(GL_DEPTH_TEST);
shibaCheckGlError();

// Pass 0 ribbons
glUseProgram(shibaPrograms[0]);
shibaCheckGlError();
glUniform1fv(shibaUniformLocations[0][shibaFloatUniformLocationIndex], shibaFloatUniformCount, shibaFloatUniforms);
shibaCheckGlError();
glDisable(GL_CULL_FACE);
shibaCheckGlError();
glBindVertexArray(vao);
shibaCheckGlError();
glDrawElements(GL_TRIANGLE_STRIP, indiceCount, GL_UNSIGNED_INT, indices);
shibaCheckGlError();

// Pass 1 particles
glUseProgram(shibaPrograms[1]);
shibaCheckGlError();
glUniform1fv(shibaUniformLocations[1][shibaFloatUniformLocationIndex], shibaFloatUniformCount, shibaFloatUniforms);
shibaCheckGlError();
glEnable(GL_CULL_FACE);
shibaCheckGlError();
glBindVertexArray(vaoParticles);
shibaCheckGlError();
glDrawElements(GL_TRIANGLE_STRIP, indiceParticleCount, GL_UNSIGNED_INT, indicesParticles);
shibaCheckGlError();

// Pass 2 post processing
glBindFramebuffer(GL_FRAMEBUFFER, shibaFinalFramebufferId);
shibaCheckGlError();
glUseProgram(shibaPrograms[2]);
shibaCheckGlError();
glUniform1fv(shibaUniformLocations[2][shibaFloatUniformLocationIndex], shibaFloatUniformCount, shibaFloatUniforms);
shibaCheckGlError();
//glDisable(GL_CULL_FACE);
//shibaCheckGlError();
glDisable(GL_DEPTH_TEST);
shibaCheckGlError();
glActiveTexture(GL_TEXTURE0 + 0);
shibaCheckGlError();
glBindTexture(GL_TEXTURE_2D, firstPassTextureId);
shibaCheckGlError();
glUniform1i(shibaUniformLocations[2][shibaSampler2DUniformLocationIndex], 0);
shibaCheckGlError();
glClear(GL_COLOR_BUFFER_BIT);
shibaCheckGlError();
shibaDrawScreenRect();
shibaCheckGlError();
