const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './client-browser/src/index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: path.resolve(__dirname, "client-browser/src/index.html")
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "client"),
            extraArgs: '--no-typescript',
            // outDir: path.resolve(__dirname, "client-browser/src/pkg")
        })
    ],
    mode: 'development',
    experiments: {
        syncWebAssembly: true
    },
    devServer: {
        static: {
            directory: path.resolve(__dirname, "client-browser/webroot")
        }
    }
};
