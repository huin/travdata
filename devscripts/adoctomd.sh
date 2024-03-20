#!/usr/bin/env bash

f="$1"
xml="${f%.*}.xml"
out="${f%.*}.md"
asciidoc -b docbook "$f"
pandoc -f docbook -t markdown_strict "$xml" -o "$out"
rm "$xml"
