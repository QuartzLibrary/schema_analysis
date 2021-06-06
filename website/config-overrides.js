const path = require('path');

module.exports = function override(config, env) {
    const wasmExtensionRegExp = /\.wasm$/;
    const workerRegExp = /\.worker\.js$/;

    config.resolve.extensions.push('.wasm');

    config.module.rules.forEach(rule => {
        (rule.oneOf || []).forEach(oneOf => {
            if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
                // Make file-loader ignore WASM files
                oneOf.exclude.push(wasmExtensionRegExp);
            }
        });
    });

    // Add a dedicated loader for WASM
    config.module.rules.push({
        test: wasmExtensionRegExp,
        include: path.resolve(__dirname, 'src'),
        use: [{ loader: require.resolve('wasm-loader'), options: {} }]
    });
    // Add a dedicated loader for WASM
    config.module.rules.push({
        test: workerRegExp,
        include: path.resolve(__dirname, 'src'),
        use: [{ loader: require.resolve('worker-loader'), options: {} }]
    });

    return config;
};