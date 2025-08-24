import init, * as bindings from '/renderer/point-cloud-wasm.js';

(async () => {
    const wasm = await init ({module_or_path: '/renderer/point-cloud-wasm_bg.wasm'});
    window.wasmBindings = bindings;
    console.log('Bevy WASM was initialized', wasm);
})();