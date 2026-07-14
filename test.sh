#!/bin/sh


#############################
#   running monkey lang tests
############################


echo "Kisumu Lang examples"
for i in $(ls src/bin/klc/examples);
do
    echo "-- running Kisumu lang examples --"
    echo $PWD/$i
done;


#  zig examples
 zig_examples() {
    for i in $(find test -type f -name *.zig);
        do
            echo "--- running zig examples ---"
            $(zig run "$i" -fno-emit-bin -femit-llvm-ir="$i".ll)
            echo $i
    done;
    mkdir -p test/output
    $(mv test/*.ll test/output/)
}

zig_examples
