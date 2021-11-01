# basic-fetch-wasm
WASM module for basic fetching of sources. This is used in the oracle validator node as default resolver. Currently it's the only resolver available, more to come later.

If you want to run this you are probably looking for https://github.com/fluxprotocol/oracle-validator-node which includes this resolver.

# Building

You should have binaryen installed (`brew install binaryen`) in order to make a optimised WASM file.