for i in *.vert *.frag; do
    glslangValidator -V $i -o "$i.spv"
done