# Nova-SHA512

The repository includes a benchmarking of the SHA-512 hash function based on [Nova](https://github.com/microsoft/Nova). This requires a gadget for the SHA-512 compression function, which is obtained from the [SHA-512 circuit](https://github.com/varunthakore/bellperson-sha512) implemented using [bellperson](https://github.com/filecoin-project/bellperson).

The SHA-512 compression function is represented as the step function $F$ within the Nova computation. It follows the format $z_{i+1} = F(z_i)$, where $z_0$ is a fixed initial value, denoted as $IV$.

## Running the example
Run the following commands.
```
cargo build --release
cargo run -r --example sha512 6
```
In the above case, The input message to SHA-512 will be $2^6$ zero bytes. The output will look like the following.
```
Nova-based SHA512 compression function iterations
=========================================================
Producing public parameters...
PublicParams::setup, took 5.919208353s 
Number of constraints per step (primary circuit): 78415
Number of constraints per step (secondary circuit): 10347
Number of variables per step (primary circuit): 78331
Number of variables per step (secondary circuit): 10329
Generating a RecursiveSNARK...
RecursiveSNARK::prove_step 0: true, took 121.632004ms 
Total time taken by RecursiveSNARK::prove_steps: 121.645998ms
Verifying a RecursiveSNARK...
RecursiveSNARK::verify: true, took 80.677392ms
Generating a CompressedSNARK using Spartan with IPA-PC...
CompressedSNARK::prove: true, took 6.328847634s
Total proving time is 8.185695442s
CompressedSNARK::len 9924 bytes
Verifying a CompressedSNARK...
CompressedSNARK::verify: true, took 268.03797ms
=========================================================
Public parameters generation time: 5.919208353s 
Total proving time (excl pp generation): 8.185695442s
Total verification time: 268.03797ms
=========================================================
Expected value of final hash = "7be9fda48f4179e611c698a73cff09faf72869431efee6eaad14de0cb44bbf66503f752b7a8eb17083355f3ce6eb7d2806f236b25af96a24e22b887405c20081"
Actual value of final hash   = "7be9fda48f4179e611c698a73cff09faf72869431efee6eaad14de0cb44bbf66503f752b7a8eb17083355f3ce6eb7d2806f236b25af96a24e22b887405c20081"
```

To change the input length to 64 KB ($2^{16}$ bytes), run the following command. The input is again all zero bytes.
```
cargo run -r --example sha512 16
```

## Generating the benchmarks
Run the following commands (shell script tested only on Ubuntu).
```bash
cargo build --release --examples
./genlog_all.sh
```
The `logs` directory will have two files per input length `N`, for `N` in the set {64, 128, ..., 65536}.

- `output_N.txt` has the program output.
- `time_output_N.txt` has the output of the `time` command. This is used to measure peak memory usage.

To generate the logs for a particular length, you can run the `genlog.sh` script. For example, the following command will generate logs for input length 1024 bytes.
```
./genlog.sh 10
```
## Existing benchmarks
The existing files in the `logs` directory were generated on a Dell Inspiron laptop with a [7th Gen Intel i7-7700 CPU](https://www.intel.in/content/www/in/en/products/sku/97128/intel-core-i77700-processor-8m-cache-up-to-4-20-ghz/specifications.html) and 8 GB of RAM. The CPU has 4 cores with 2 threads per core.
- For all lengths
  - The peak memory usage was about 445 MB.
  - Verification time was less than 235 milliseconds.
  - Proof size was about 10,000 bytes.
  - Public parameter generation time was about 5.6 seconds
- The proving time for 64KB input was less than 2 minutes. Proving times for other lengths are shown below.

### Proving times
```bash
$ grep "Total proving time is" $(ls logs/output_* -rt)
logs/output_65536.txt:Total proving time is 106.04168828s
logs/output_32768.txt:Total proving time is 56.704587013s
logs/output_16384.txt:Total proving time is 32.900125257s
logs/output_8192.txt:Total proving time is 20.220806356s
logs/output_4096.txt:Total proving time is 14.238238058s
logs/output_2048.txt:Total proving time is 11.310179391s
logs/output_1024.txt:Total proving time is 9.774121711s
logs/output_512.txt:Total proving time is 8.969354571s
logs/output_256.txt:Total proving time is 8.675470891s
logs/output_128.txt:Total proving time is 8.363046923s
logs/output_64.txt:Total proving time is 8.069623865s
```

### Verification times
```bash
$ grep "CompressedSNARK::verify" $(ls logs/output_* -rt)
logs/output_65536.txt:CompressedSNARK::verify: true, took 232.895295ms
logs/output_32768.txt:CompressedSNARK::verify: true, took 215.429707ms
logs/output_16384.txt:CompressedSNARK::verify: true, took 230.572868ms
logs/output_8192.txt:CompressedSNARK::verify: true, took 229.04544ms
logs/output_4096.txt:CompressedSNARK::verify: true, took 231.415676ms
logs/output_2048.txt:CompressedSNARK::verify: true, took 240.015387ms
logs/output_1024.txt:CompressedSNARK::verify: true, took 227.635137ms
logs/output_512.txt:CompressedSNARK::verify: true, took 228.219729ms
logs/output_256.txt:CompressedSNARK::verify: true, took 226.233019ms
logs/output_128.txt:CompressedSNARK::verify: true, took 212.494493ms
logs/output_64.txt:CompressedSNARK::verify: true, took 226.892958ms
```

### Proof sizes
```bash
$ grep "len" $(ls logs/output_* -rt)
logs/output_65536.txt:CompressedSNARK::len 10401 bytes
logs/output_32768.txt:CompressedSNARK::len 10399 bytes
logs/output_16384.txt:CompressedSNARK::len 10408 bytes
logs/output_8192.txt:CompressedSNARK::len 10409 bytes
logs/output_4096.txt:CompressedSNARK::len 10400 bytes
logs/output_2048.txt:CompressedSNARK::len 10399 bytes
logs/output_1024.txt:CompressedSNARK::len 10401 bytes
logs/output_512.txt:CompressedSNARK::len 10400 bytes
logs/output_256.txt:CompressedSNARK::len 10403 bytes
logs/output_128.txt:CompressedSNARK::len 10370 bytes
logs/output_64.txt:CompressedSNARK::len 9924 bytes
```

### Peak memory usage
```bash
$ grep "Maximum resident set size" $(ls logs/time_output_* -rt)
logs/time_output_65536.txt:     Maximum resident set size (kbytes): 415852
logs/time_output_32768.txt:     Maximum resident set size (kbytes): 406892
logs/time_output_16384.txt:     Maximum resident set size (kbytes): 419828
logs/time_output_8192.txt:      Maximum resident set size (kbytes): 419848
logs/time_output_4096.txt:      Maximum resident set size (kbytes): 415584
logs/time_output_2048.txt:      Maximum resident set size (kbytes): 414428
logs/time_output_1024.txt:      Maximum resident set size (kbytes): 408232
logs/time_output_512.txt:       Maximum resident set size (kbytes): 408944
logs/time_output_256.txt:       Maximum resident set size (kbytes): 406816
logs/time_output_128.txt:       Maximum resident set size (kbytes): 417444
logs/time_output_64.txt:        Maximum resident set size (kbytes): 445028
```
### Public parameter generation time
```bash
$ grep "Public parameters" $(ls logs/output_* -rt)
logs/output_65536.txt:Public parameters generation time: 5.665662432s 
logs/output_32768.txt:Public parameters generation time: 5.646133304s 
logs/output_16384.txt:Public parameters generation time: 5.630218268s 
logs/output_8192.txt:Public parameters generation time: 5.642008471s 
logs/output_4096.txt:Public parameters generation time: 5.634555935s 
logs/output_2048.txt:Public parameters generation time: 5.65060872s 
logs/output_1024.txt:Public parameters generation time: 5.631631182s 
logs/output_512.txt:Public parameters generation time: 5.663318608s 
logs/output_256.txt:Public parameters generation time: 5.62858415s 
logs/output_128.txt:Public parameters generation time: 5.632879184s 
logs/output_64.txt:Public parameters generation time: 5.62004281s 
```

## Acknowledgement
This project is adapted from [Prof Saravanan Vijayakumaran's](https://www.ee.iitb.ac.in/~sarva/) benchmarking of [Nova-based SHA256](https://github.com/avras/nova-sha256).
