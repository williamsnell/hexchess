#! /bin/bash

npm install
npm run build
# kill the existing process
killall -9 node
# sleep in case killall takes a while?
sleep .5
# start the new process
node build/index.js > ./err_f.txt 2>&1 &
