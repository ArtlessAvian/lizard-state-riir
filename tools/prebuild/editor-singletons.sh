#!/bin/bash

# Thanks to https://forum.godotengine.org/t/how-to-strip-editor-specific-code-from-build-in-godot-4-2/41978/4

set -eux

editor_only_classes=(
    EditorInterface
)

for i in "${editor_only_classes[@]}"
do
    find godot -iname *.gd | xargs sed -i s"/$i\./Engine.get_singleton(\"$i\")\./"
done
