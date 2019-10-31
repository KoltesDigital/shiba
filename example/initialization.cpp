// ribbons
auto pVertices = vertices;
auto pIndices = indices;
for (int index = 0; index < count; ++index)
{
	for (int y = 0; y < faceY; ++y)
	{
		for (int x = 0; x < faceX; ++x)
		{
			*(pVertices++) = (float)index;
			*(pVertices++) = (float)index / (float)count;
			*(pVertices++) = (float)index;
			*(pVertices++) = (float)x / (float)sliceX;
			*(pVertices++) = (float)y / (float)sliceY;
		}
	}

	for (int r = 0; r < faceY - 1; ++r)
	{
		*(pIndices++) = index * (faceX * faceY) + r * faceX;
		for (int c = 0; c < faceX; ++c)
		{
			*(pIndices++) = index * (faceX * faceY) + r * faceX + c;
			*(pIndices++) = index * (faceX * faceY) + (r + 1) * faceX + c;
		}
		*(pIndices++) = index * (faceX * faceY) + (r + 1) * faceX + (faceX - 1);
	}
}

// particles
auto pVerticesParticle = verticesParticles;
auto pIndicesParticle = indicesParticles;
for (int index = 0; index < particleCount; ++index)
{
	for (int y = 0; y < 2; ++y)
	{
		for (int x = 0; x < 2; ++x)
		{
			*(pVerticesParticle++) = (float)index;
			*(pVerticesParticle++) = (float)index / (float)particleCount;
			*(pVerticesParticle++) = (float)index;
			*(pVerticesParticle++) = ((float)x) * 2.f - 1.f;
			*(pVerticesParticle++) = ((float)y) * 2.f - 1.f;
		}
	}
	for (int r = 0; r < 2 - 1; ++r)
	{
		*(pIndicesParticle++) = index * (2 * 2) + r * 2;
		for (int c = 0; c < 2; ++c)
		{
			*(pIndicesParticle++) = index * (2 * 2) + r * 2 + c;
			*(pIndicesParticle++) = index * (2 * 2) + (r + 1) * 2 + c;
		}
		*(pIndicesParticle++) = index * (2 * 2) + (r + 1) * 2 + (2 - 1);
	}
}

// ribbons
glCreateBuffers(1, &vbo);
shibaCheckGlError();
glNamedBufferStorage(vbo, sizeof(vertices), vertices, 0);
shibaCheckGlError();
glCreateVertexArrays(1, &vao);
shibaCheckGlError();
glBindVertexArray(vao);
shibaCheckGlError();
glBindBuffer(GL_ARRAY_BUFFER, vbo);
shibaCheckGlError();
glEnableVertexAttribArray(0);
shibaCheckGlError();
glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 5 * sizeof(GLfloat), 0);
shibaCheckGlError();
glEnableVertexAttribArray(1);
shibaCheckGlError();
glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 5 * sizeof(GLfloat), (GLvoid *)(3 * sizeof(GLfloat)));
shibaCheckGlError();

// particles
glCreateBuffers(1, &vboParticles);
shibaCheckGlError();
glNamedBufferStorage(vboParticles, sizeof(verticesParticles), verticesParticles, 0);
shibaCheckGlError();
glCreateVertexArrays(1, &vaoParticles);
shibaCheckGlError();
glBindVertexArray(vaoParticles);
shibaCheckGlError();
glBindBuffer(GL_ARRAY_BUFFER, vboParticles);
shibaCheckGlError();
glEnableVertexAttribArray(0);
shibaCheckGlError();
glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 5 * sizeof(GLfloat), 0);
shibaCheckGlError();
glEnableVertexAttribArray(1);
shibaCheckGlError();
glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 5 * sizeof(GLfloat), (GLvoid *)(3 * sizeof(GLfloat)));
shibaCheckGlError();

glGenTextures(1, &firstPassTextureId);
shibaCheckGlError();
glBindTexture(GL_TEXTURE_2D, firstPassTextureId);
shibaCheckGlError();
glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA32F, shibaResolutionWidth, shibaResolutionHeight, 0, GL_RGBA, GL_FLOAT, NULL);
shibaCheckGlError();
glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
shibaCheckGlError();
//glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
//shibaCheckGlError();
glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
shibaCheckGlError();
glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);

glGenFramebuffers(1, &fbo);
shibaCheckGlError();
