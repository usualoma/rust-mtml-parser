const fs = require("fs");
const path = require("path");

function findFiles(dir, exts, callback) {
  let files = fs.readdirSync(dir);
  files.filter(file => fs.statSync(path.join(dir, file)).isDirectory()).forEach(subdir => {
    findFiles(path.join(dir, subdir), exts, callback);
  })
  files = files.filter(file => exts.includes(path.extname(file)));
  callback(null, files.map(file => path.join(dir, file)));
}
const allFiles = [];
findFiles("pkg", [".wasm", ".js", ".ts"], (err, files) => {
  if (err) {
    throw err;
  }
  allFiles.push(...files);
});

const packageJson = JSON.parse(fs.readFileSync("pkg/package.json", "utf8"));
packageJson.name = "mtml-parser";
packageJson.files = allFiles.map(file => file.replace("pkg/", ""));
fs.writeFileSync("pkg/package.json", JSON.stringify(packageJson, null, 2));
