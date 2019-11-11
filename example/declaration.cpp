SHIBA_VARIABLE GLuint vbo, vao, vboParticles, vaoParticles;

SHIBA_CONST int count = 1;
SHIBA_CONST int sliceX = 1000;
SHIBA_CONST int sliceY = 1;
SHIBA_CONST int faceX = sliceX + 1;
SHIBA_CONST int faceY = sliceY + 1;
SHIBA_CONST int indiceCount = count * ((faceX + 2) * faceY);	 // count * line (+2 obfuscated triangle) * row
SHIBA_CONST int vertexCount = count * (faceX * faceY) * (3 + 2); // count * line * row * (x,y,z + u,v)
SHIBA_CONST int particleCount = 200;
SHIBA_CONST int indiceParticleCount = particleCount * 4 * 6;

SHIBA_VARIABLE GLfloat vertices[vertexCount];
SHIBA_VARIABLE GLfloat verticesParticles[particleCount * 6 * 4];
SHIBA_VARIABLE unsigned int indices[indiceCount];
SHIBA_VARIABLE unsigned int indicesParticles[indiceParticleCount];

SHIBA_VARIABLE GLuint firstPassTextureId;

SHIBA_VARIABLE unsigned int fbo;
