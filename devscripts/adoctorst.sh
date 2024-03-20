#!/usr/bin/env bash

f="$1"
xml="${f%.*}.xml"
out="${f%.*}.rst"
asciidoc -b docbook "$f"
pandoc -f docbook -t rst "$xml" -o "$out"
rm "$xml"
