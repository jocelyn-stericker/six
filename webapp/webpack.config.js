const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const BundleAnalyzerPlugin = require("webpack-bundle-analyzer")
  .BundleAnalyzerPlugin;
const { NormalModuleReplacementPlugin } = require("webpack");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  resolve: {
    extensions: [".ts", ".tsx", ".js", ".jsx"]
  },
  module: {
    rules: [
      {
        test: /\.(eot|ttf|woff|woff2|svg|png|gif|jpe?g)$/,
        use: [{ loader: "file-loader" }]
      },
      {
        test: /\.css$/i,
        use: ["style-loader", "css-loader"]
      },
      {
        test: /\.m?[jt]sx?$/,
        exclude: /(node_modules|bower_components)/,
        use: {
          loader: "babel-loader",
          options: {
            presets: [
              "@babel/preset-env",
              "@babel/preset-react",
              "@babel/preset-typescript"
            ]
          }
        }
      }
    ]
  },
  entry: {
    index: "./src/index.tsx"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    contentBase: dist
  },
  plugins: [
    new CopyPlugin([path.resolve(__dirname, "static")]),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "..", "render"),
      outDir: path.resolve(__dirname, "rust_render_built")
    }),

    new NormalModuleReplacementPlugin(
      /.*\/generated\/iconSvgPaths.*/,
      path.resolve(__dirname, "src/blueprint/icons.js")
    ),

    new NormalModuleReplacementPlugin(
      /.*dom4.*/,
      path.resolve(__dirname, "src/blueprint/blank.js")
    )

    // new BundleAnalyzerPlugin()
  ].filter(a => !!a)
};
