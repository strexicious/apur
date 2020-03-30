@echo off
for %%s in (
    shader.vert
    shader.frag
    skybox.vert
    skybox.frag
) do (
    glslangValidator -V %%s -o %%s.spv
)
