#!/bin/bash
cd $(dirname $0)
cd ..
dot src/iris1.dot -Tsvg -odoc/images/iris1.svg
dot src/iris2.dot -Tsvg -odoc/images/iris2.svg
dot src/iris3.dot -Tsvg -odoc/images/iris3.svg
dot examples/example.dot -Tsvg -odoc/images/tree_example.svg
