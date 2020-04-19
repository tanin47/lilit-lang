#!/bin/bash

echo "TO STDOUT"
echo "TO STDERR" 1>&2;

printf "Please enter your name: "
read name

echo "HELLO $name"