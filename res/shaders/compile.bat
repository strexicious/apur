@echo off
for %%s in (
    solid.vert
    solid.frag
    transparent.vert
    transparent.frag
    combined.vert
    combined.frag
    collatz.comp
) do (
    glslangValidator -V %%s -o %%s.spv
)
