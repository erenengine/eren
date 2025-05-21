const path = require('path');
const webpack = require('webpack');

module.exports = {
  entry: {
    'test-sprite': './tests/test-sprite/index.ts',
  },
  output: {
    filename: '[name]/bundle.js',
    path: path.resolve(__dirname, 'tests')
  },
  plugins: [
    new webpack.optimize.LimitChunkCountPlugin({
      maxChunks: 1
    })
  ],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  devServer: {
    client: {
      logging: 'none',
    },
  },
};