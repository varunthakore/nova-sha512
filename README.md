# Nova-SHA512

The repository includes a benchmarking of the SHA-512 hash function based on [Nova](https://github.com/microsoft/Nova). This requires a gadget for the SHA-512 compression function, which is obtained from the [SHA-512 circuit](https://github.com/lurk-lab/bellpepper-gadgets/tree/main/crates/sha512) implemented using [bellpepper](https://github.com/lurk-lab/bellpepper).

The SHA-512 compression function is represented as the step function $F$ within the Nova computation. It follows the format $z_{i+1} = F(z_i)$, where $z_0$ is a fixed initial value, denoted as $IV$.

## Running the example
Run the following commands.
```
cargo build --release
cargo run --release --example sha512 6
```
In the above case, The input message to SHA-512 will be $2^6$ zero bytes. The output will look like the following.
```
Nova-based SHA512 compression function iterations
=========================================================
Producing public parameters...
PublicParams::setup, took 5.414872083s 
Number of constraints per step (primary circuit): 78415
Number of constraints per step (secondary circuit): 10347
Number of variables per step (primary circuit): 78331
Number of variables per step (secondary circuit): 10329
Generating a RecursiveSNARK...
RecursiveSNARK::prove_step 0: true, took 4.333µs 
Total time taken by RecursiveSNARK::prove_steps: 27.458µs
Verifying a RecursiveSNARK...
RecursiveSNARK::verify: true, took 49.15675ms
Generating a CompressedSNARK using Spartan with IPA-PC...
CompressedSNARK::prove: true, took 7.57205325s
Total proving time is 8.230779375s
CompressedSNARK::len 9913 bytes
Verifying a CompressedSNARK...
CompressedSNARK::verify: true, took 309.657833ms
=========================================================
Public parameters generation time: 5.414872083s 
Total proving time (excl pp generation): 8.230779375s
Total verification time: 309.657833ms
=========================================================
Expected value of final hash = "7be9fda48f4179e611c698a73cff09faf72869431efee6eaad14de0cb44bbf66503f752b7a8eb17083355f3ce6eb7d2806f236b25af96a24e22b887405c20081"
Actual value of final hash   = "7be9fda48f4179e611c698a73cff09faf72869431efee6eaad14de0cb44bbf66503f752b7a8eb17083355f3ce6eb7d2806f236b25af96a24e22b887405c20081"
```

To change the input length to 64 KB ($2^{16}$ bytes), run the following command. The input is again all zero bytes.
```
cargo run --release --example sha512 16
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
The existing files in the `logs` directory were generated on Apple Macbook Air M1 with 8GB RAM.
- For all lengths
  - The peak memory usage was about 522 MB.
  - Verification time was less than 260 milliseconds.
  - Proof size was about 10,000 bytes.
  - Public parameter generation time was about 5.3 seconds
- The proving time for 64KB input was just over 100s. Proving times for other lengths are shown below.

### Proving times
```bash
$ grep "Total proving time is" $(ls -rt logs/output_*)
logs/output_65536.txt:Total proving time is 104.831938083s
logs/output_32768.txt:Total proving time is 59.466137458s
logs/output_16384.txt:Total proving time is 31.542905375s
logs/output_8192.txt:Total proving time is 19.869245042s
logs/output_4096.txt:Total proving time is 14.004221375s
logs/output_2048.txt:Total proving time is 11.19301675s
logs/output_1024.txt:Total proving time is 9.965042583s
logs/output_512.txt:Total proving time is 10.307841959s
logs/output_256.txt:Total proving time is 10.498092333s
logs/output_128.txt:Total proving time is 8.687329542s
logs/output_64.txt:Total proving time is 8.20162325s
```

### Verification times
```bash
$ grep "CompressedSNARK::verify" $(ls -rt logs/output_*)
logs/output_65536.txt:CompressedSNARK::verify: true, took 248.697125ms
logs/output_32768.txt:CompressedSNARK::verify: true, took 266.471667ms
logs/output_16384.txt:CompressedSNARK::verify: true, took 248.045542ms
logs/output_8192.txt:CompressedSNARK::verify: true, took 247.234416ms
logs/output_4096.txt:CompressedSNARK::verify: true, took 249.306458ms
logs/output_2048.txt:CompressedSNARK::verify: true, took 246.204209ms
logs/output_1024.txt:CompressedSNARK::verify: true, took 286.810875ms
logs/output_512.txt:CompressedSNARK::verify: true, took 294.566292ms
logs/output_256.txt:CompressedSNARK::verify: true, took 256.616ms
logs/output_128.txt:CompressedSNARK::verify: true, took 249.140125ms
logs/output_64.txt:CompressedSNARK::verify: true, took 259.960375ms
```

### Proof sizes
```bash
$ grep "len" $(ls -rt logs/output_*)
logs/output_65536.txt:CompressedSNARK::len 10297 bytes
logs/output_32768.txt:CompressedSNARK::len 10302 bytes
logs/output_16384.txt:CompressedSNARK::len 10302 bytes
logs/output_8192.txt:CompressedSNARK::len 10303 bytes
logs/output_4096.txt:CompressedSNARK::len 10298 bytes
logs/output_2048.txt:CompressedSNARK::len 10302 bytes
logs/output_1024.txt:CompressedSNARK::len 10300 bytes
logs/output_512.txt:CompressedSNARK::len 10303 bytes
logs/output_256.txt:CompressedSNARK::len 10296 bytes
logs/output_128.txt:CompressedSNARK::len 10263 bytes
logs/output_64.txt:CompressedSNARK::len 9913 bytes
```

### Peak memory usage in bytes
```bash
$ grep "maximum resident set size" $(ls -rt logs/time_output_*)
logs/time_output_65536.txt:           401932288  maximum resident set size
logs/time_output_32768.txt:           425590784  maximum resident set size
logs/time_output_16384.txt:           422412288  maximum resident set size
logs/time_output_8192.txt:           460095488  maximum resident set size
logs/time_output_4096.txt:           460521472  maximum resident set size
logs/time_output_2048.txt:           460259328  maximum resident set size
logs/time_output_1024.txt:           474382336  maximum resident set size
logs/time_output_512.txt:           418037760  maximum resident set size
logs/time_output_256.txt:           413073408  maximum resident set size
logs/time_output_128.txt:           521895936  maximum resident set size
logs/time_output_64.txt:           499105792  maximum resident set size
```
### Public parameter generation time
```bash
$ grep "Public parameters" $(ls -rt logs/output_*)
logs/output_65536.txt:Public parameters generation time: 5.272849542s 
logs/output_32768.txt:Public parameters generation time: 5.451450625s 
logs/output_16384.txt:Public parameters generation time: 5.496126208s 
logs/output_8192.txt:Public parameters generation time: 5.288794292s 
logs/output_4096.txt:Public parameters generation time: 5.308094333s 
logs/output_2048.txt:Public parameters generation time: 5.298190833s 
logs/output_1024.txt:Public parameters generation time: 5.320729542s 
logs/output_512.txt:Public parameters generation time: 5.725966625s 
logs/output_256.txt:Public parameters generation time: 5.858633417s 
logs/output_128.txt:Public parameters generation time: 5.33493275s 
logs/output_64.txt:Public parameters generation time: 5.343597s  
```

This project was part of the submission for [ZK MOOC Hackathon](https://zk-hacking.org/) for [zk-Circuits Track](https://zk-hacking.org/tracks/zk_circuit_track/) and it was selected as one of the winners for *Category 2: Circuits/R1CSs for recursive SNARKs*.

## Acknowledgement
This project is adapted from [Prof Saravanan Vijayakumaran's](https://www.ee.iitb.ac.in/~sarva/) benchmarking of [Nova-based SHA256](https://github.com/avras/nova-sha256).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
