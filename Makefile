build-frontend:
	cd frontend && trunk build --release

build-backend:
	cargo build -p backend --release

build-production: build-frontend build-backend

run-production:
	cargo run -p backend --release
