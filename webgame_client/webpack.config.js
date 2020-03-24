const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const distPath = path.resolve(__dirname, "dist");
module.exports = (env, argv) => {
  return {
    devServer: {
      contentBase: distPath,
      compress: argv.mode === 'production',
      host: '0.0.0.0',
      port: 8001,
      proxy: {
        "/ws": {
          target: "http://127.0.0.1:8002",
          changeOrigin: true,
          ws: true,
        }
      },
    },
    entry: './bootstrap.js',
    output: {
      path: distPath,
      filename: "webgame.js",
      webassemblyModuleFilename: "webgame.wasm",
    },
    plugins: [
      new CopyWebpackPlugin([{
        from: './static',
        to: distPath
      }, ]),
      new WasmPackPlugin({
        crateDirectory: ".",
        extraArgs: "--no-typescript",
      }),
    ],
    watch: argv.mode !== 'production',
  };
};