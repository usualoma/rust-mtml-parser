build: clean
	wasm-pack build --target nodejs --release --out-dir pkg && \
	wasm-pack build --target bundler --release --out-dir pkg/bundler && \
	wasm-pack build --target web --release --out-dir pkg/web && \
	rm pkg/bundler/package.json pkg/web/package.json

clean:
	rm -rf pkg
