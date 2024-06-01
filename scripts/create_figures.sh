#!/bin/bash
cd $(dirname $0)
cd ..
dot src/iris1.dot -Tsvg -odoc/images/iris1.svg
