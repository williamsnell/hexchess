#! /bin/bash

cd ~/hexchess/frontend
npm run build
# kill the existing process
killall -9 node
# start the new process
node build/index.js > ./err_f.txt 2>&1 &