const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "public"),
    filename: "main.js",
  },
  mode: "development",
  experiments: {
    syncWebAssembly: true,
  },
};
