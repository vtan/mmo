const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './client/src/index.js',
    output: {
        path: path.resolve(__dirname, 'target/dist/webroot'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: path.resolve(__dirname, "client/src/index.html")
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "client"),
            extraArgs: '--no-typescript',
            forceMode: process.env.NODE_ENV === 'production' ? 'release' : undefined
        })
    ],
    mode: process.env.NODE_ENV || 'development',
    experiments: {
        asyncWebAssembly: true
    },
    devServer: {
        proxy: {
            "/api/ws": {
                target: "ws://localhost:8081",
                ws: true
            },
            "/": {
                target: "http://localhost:8081"
            }
        }
    }
};
