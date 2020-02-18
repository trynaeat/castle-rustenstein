release:
		cargo build --release
		cp -r data target/release
		cp *.dll target/release

clean:
		rm -rf target/release