.PHONY: bench_native.so

bench_native.so: benchmark.c
	clang -O3 -fPIC -I. -L../../../../engines/native/  -DNATIVE_ENGINE -shared -o bench_native.so benchmark.c -lengine
	mv bench_native.so ../../