const path = require("path");

module.exports = {
	mode: "production",
	entry: path.resolve(__dirname, "main.ts"),
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "ts.bundle.js"
	}
};
