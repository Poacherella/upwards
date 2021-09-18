wasm-pack build --target web --release
wasm-opt pkg/upwards_bg.wasm -o pkg/upwards_bg.wasm -O3
cp pkg/upwards_bg.wasm web/
cp pkg/upwards.js web/