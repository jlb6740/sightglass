.PHONY: bench_native.so

bench_native.so: benchmark.c
        clang -O3 -fPIC -I. -L../../../../engines/native/ \
                -DNATIVE_ENGINE \
                -msimd128 -msse2 -msse3 -msse4.1 -msse4.2 \
                -DBLAKE3_NO_SSE41 -DBLAKE3_NO_AVX2 -DBLAKE3_NO_AVX512 \
                -shared -o bench_native.so \
                blake3.c blake3_sse2.c blake3_dispatch.c \
                blake3_portable.c benchmark.c -lengine
        mv bench_native.so ../../