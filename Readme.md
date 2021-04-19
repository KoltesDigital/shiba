# Shiba

**Work in progress! This is not stable yet. Expect many breaking changes.**

Shiba is a digital content creation tool which aims at:

1. developing in real-time -> final image in Blender viewport with hot reload of sources;
2. being able to produce demos (see [demoscene](https://en.wikipedia.org/wiki/Demoscene)) -> supports GLSL, C++, and therefore custom compositing with framebuffers, etc.;
3. easily animating stuff -> use Blender animation features;
4. supporting live coding performances -> feature in progress.

## Prerequisites
- [Visual Studio 2019](https://visualstudio.microsoft.com/downloads/)
- [Python 2.7](https://www.python.org/downloads/release/python-2710/)
- [Glew](http://glew.sourceforge.net/)
- [Shiba Binary](https://www.dropbox.com/sh/fakx97rntvm24bw/AABp3vRu4FaMquj1x2wWi7K8a?dl=0)

## config.yml
To be placed in folder 'C:\Users\[UserName]\.shiba\config.yml'
  ```yml
  paths:
    glew: "C:\\Tools\\glew-2.1.0"
    python2: "C:\\Python27\\python.exe"
  ```

## Project Examples

- [PiaggioNonTroppo](https://github.com/KoltesDigital/shiba-piaggio-non-troppo)
- [Raymarching](https://github.com/KoltesDigital/shiba-raymarching)
- [Shadertoy template](https://github.com/KoltesDigital/shiba-shadertoy-template)
