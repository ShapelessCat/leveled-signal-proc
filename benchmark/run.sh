#!/bin/zsh
set -x
#rm -f {poc,tlb2}-{playtime,cir}.txt
for ((i=0;i<5;i++))
do
	#../lsp/target/release/examples/tlb2-dag >> poc-playtime.txt
	../lsp/target/release/examples/cir-dag >> poc-cir.txt
	#pushd ..
	#../timeline-rust/target/release/batch playtime.yaml >> benchmark/tlb2-playtime.txt
	#../timeline-rust/target/release/batch cir.yaml >> benchmark/tlb2-cir.txt
	#popd
done
