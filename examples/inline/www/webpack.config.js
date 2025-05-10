const path = require("node:path");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./index.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
  },
  devServer: {
    static: {
      directory: dist,
    },
  },
  performance: {
    hints: "warning",
    maxAssetSize: 500 * 1024,
    maxEntrypointSize: 500 * 1024,
    assetFilter: (assetFilename) => {
      return !/\.wasm$/.test(assetFilename);
    },
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: ["static/index.html"],
    }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".."),
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: ["style-loader", "css-loader"],
      },
      {
        test: /\.wasm$/,
        type: "webassembly/async",
      },
    ],
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
