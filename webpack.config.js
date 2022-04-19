const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
  entry: "./client-browser/src/index.ts",

  output: {
    filename: "bundle.js",
    path: __dirname + "/target/client-browser/"
  },

  devtool: "source-map",

  resolve: {
    extensions: [".ts", ".tsx", ".js", ".json"]
  },

  module: {
    rules: [
      { test: /\.tsx?$/, loader: "ts-loader" },
      { enforce: "pre", test: /\.js$/, loader: "source-map-loader" }
    ],
  },

  plugins: [
    new HtmlWebpackPlugin({
      "template": "client-browser/src/index.html"
    })
  ],

  devServer: {
    static: {
      directory: "client-browser/webroot/"
    },
    proxy: {
      "/api": {
        target: "http://0.0.0.0:8081"
      }
    }
  }
};
