#!/bin/bash
#runs from the root of the repo folder
echo 'inside post create command'

cp .devcontainer/.bash_aliases ~/

source ~/.bashrc

echo 'finished post create command'