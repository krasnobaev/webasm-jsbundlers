#!/bin/bash
FILE=./dist/index.js
echo 'window.global = window;' | cat - $FILE > $FILE.tmp && mv $FILE.tmp $FILE
sed -i '' 's/ Buffer\./ Buffer$1\./g' ./dist/index.js
