#!/usr/bin/env bash
# Converts asciidoc files listed as parameters into Markdown and
# reStructuredText.

set -e

function convert_file() {
    f="$1"
    xml="${f%.*}.xml"
    md="${f%.*}.md"
    rst="${f%.*}.rst"

    asciidoc -b docbook "$f"
    pandoc -f docbook -t markdown_strict "$xml" -o "$md"
    pandoc -f docbook -t rst "$xml" -o "$rst"
    rm "$xml"
}

for file in $*; do
    convert_file "$file"
done
