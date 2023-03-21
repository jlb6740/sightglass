shootout_benchmarks=(ackermann base64 ctype ed25519 fib2 gimli heapsort keccak matrix memmove minicsv nestedloop random ratelimit seqhash sieve switch xblabla20 xchacha20)
#shootout_benchmarks=(ackermann)
summary_file_wasm=xi_data_wasm.csv
summary_file_native=xi_data_native.csv

for benchmark in "${shootout_benchmarks[@]}"
do
    echo ""
    echo ""
    #echo "Run Wasmtime"
    #echo "------------"
    #cargo run -- benchmark --engine engines/wasmtime/libengine.so --raw --output-format csv -- benchmarks/shootout-${benchmark}/benchmark.wasm > ${benchmark}_wasm.csv
    #cargo run -- summarize --input-format csv --output-format csv -f ${benchmark}_wasm.csv >> ${summary_file_wasm}
    echo ""
    echo ""
    echo "Run Native"
    echo "----------"
    cd /home/jlbirch/sightglass-jlb6740/benchmarks/shootout-${benchmark}/
    LD_LIBRARY_PATH=/home/jlbirch/sightglass-jlb6740/engines/native/:/home/jlbirch/sightglass-jlb6740/benchmarks/shootout-${benchmark}/ cargo run -- benchmark --engine /home/jlbirch/sightglass-jlb6740/engines/native/libengine.so --processes=1 --raw --output-format csv -- /home/jlbirch/sightglass-jlb6740/benchmarks/shootout-${benchmark}/benchmark.wasm > /home/jlbirch/sightglass-jlb6740/${benchmark}_native.csv
    sed '/arch\|x86_64/!d' /home/jlbirch/sightglass-jlb6740/${benchmark}_native.csv -i
    cargo run -- summarize --input-format csv --output-format csv -f /home/jlbirch/sightglass-jlb6740/${benchmark}_native.csv >> /home/jlbirch/sightglass-jlb6740/${summary_file_native}
    cd -
done