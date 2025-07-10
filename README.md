# ZisK Pessimistic Proof

## Build the client ELF File

To build the `pp-client` ELF file, run the following commands:

```bash
cd bin/client
cargo-zisk build --release
```

This will generate the ELF file at the following path:  
`./target/release/pp-client`

## Execute

Inside the `inputs` folder, you will find sample input files for pessimistic proof. To execute a specific input file in the ZisK emulator, run one of the following commands:

```bash
cd bin/client
cargo-zisk run --release -i ../../inputs/pp_input_10_10.bin
```

or

```bash
cd bin/client
ziskemu -e target/riscv64ima-zisk-zkvm-elf/release/pp-client -i ../../inputs/pp_input_10_10.bin
```

## Generate Input Block Files

To generate additional input files, you can use the `pp-input-gen` tool. For example, to generate an input file for 50 bridge exists and 50 imported bridge exits , run:

```bash
cargo run --release --bin=pp-input-gen -- -e 50 -i 50
```

The command will create a file named `pp_input_xxx_yyy.bin` in the `inputs` folder (by default), where:
- `xxx` is the number of bridge exits
- `yyy` is the number of imported bridge exits

To specify a different output folder, use the `-o` flag.