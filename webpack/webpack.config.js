const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const webpack = require('webpack');

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'index.js',
  },
  devServer: {
    contentBase: path.resolve(__dirname, 'dist'),
    host: '0.0.0.0',
    // useLocalIp: true,
    // disableHostCheck: true,
  },
  module: {
    rules: [{
      test: /\.(sa|sc|c)ss$/,
      include: path.resolve(__dirname, './'),
      loaders: [
        require.resolve('style-loader'),
        require.resolve('css-loader'),
        require.resolve('sass-loader')
      ]
    }]
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'index.html'
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '.')
    }),
    new webpack.ProvidePlugin({
      '$': 'jquery'
    }),
  ],
  mode: 'development'
};
