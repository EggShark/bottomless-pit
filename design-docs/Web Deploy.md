1. Compile your BP project like `cargo b --target wasm32-unknown-unkown`
2. use wasm-bindgen on that binary like `wasm-bindgen path_to_bin --target web --out-dir path --no-typescript` the no type script is option just depends how you have your website set up
3. make an html file with this general layout
```html
    <script type="module">
        // replace with the path to the actual example
        import init from '../examples/binary.js';
        async function run() {
            await init();
        }

        run();
    </script>
```