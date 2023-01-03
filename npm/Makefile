build: clean
	wasm-pack build --target nodejs --release --out-dir pkg && \
	wasm-pack build --target bundler --release --out-name mtml-parser --out-dir pkg/bundler && \
	wasm-pack build --target web --release --out-name mtml-parser --out-dir pkg/web && \
	rm pkg/bundler/package.json pkg/web/package.json
	node ./scripts/update-package-json.js

clean:
	rm -rf pkg
