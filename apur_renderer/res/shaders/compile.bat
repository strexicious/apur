@echo off
for %%s in (
    fixed_albedo.vert
    fixed_albedo.frag
    specular.vert
    specular.frag
    diffuse.vert
    diffuse.frag
    combined.vert
    combined.frag
) do (
    glslangValidator -V %%s -o %%s.spv
)
