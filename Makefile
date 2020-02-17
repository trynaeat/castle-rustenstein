release : cargo-release
		cp -r data target/release
		cp *.dll target/release

cargo-release :
	cargo build --release

clean:
		rm -rf target/release