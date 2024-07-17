#! /bin/bash

npm install
npm run build
# start the new process
node build/index.js > ./err_f.txt 2>&1 &
