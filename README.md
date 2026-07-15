A simple brainfuck (bf) compiler <br>
it converts bf into win64 assembly and then uses [nasm](https://github.com/netwide-assembler/nasm) to compile it into an object
and lld-link [llvm](https://github.com/llvm/llvm-project)'s linker to link it into an executable
