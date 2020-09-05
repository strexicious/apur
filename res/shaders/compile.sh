for i in */*.vert */*.frag */*.comp; do
    glslangValidator -V $i -o "$i.spv"
done