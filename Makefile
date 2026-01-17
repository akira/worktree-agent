.PHONY: build dashboard clean

build: dashboard
	cargo build --release

dashboard:
	cd dashboard && npm install && npm run build

clean:
	rm -rf dashboard/dist target
