const path = require('path');

module.exports = {
  entry: './dist/index.js',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.wgsl/,
        loader: 'webpack-wgsl-loader'
      }
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, '.'),
  },
  devServer: {
    client: {
      logging: 'none',
    },
  },
};