const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './client/src/index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: path.resolve(__dirname, "client/src/index.html")
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "client"),
            extraArgs: '--no-typescript'
        })
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
    },
    devServer: {
        static: {
            directory: path.resolve(__dirname, "client/webroot")
        },
        proxy: {
            "/api/ws": {
                target: "ws://localhost:8081",
                ws: true
            },
            "/assets/": {
                target: "http://localhost:8081"
            }
        }
    }
};
