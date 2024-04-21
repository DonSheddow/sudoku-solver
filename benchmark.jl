using Statistics, DataFrames, CSV

function bench(n)
    samples = []
    for _ in 1:n
        t = @elapsed run(pipeline(`taskset -c 0 ./target/release/sudoku-solver`, stdin="sudokus/hardest.txt"))
        push!(samples, t)
    end
    return samples
end

function compile(flags)
    run(addenv(`cargo build -r`, "RUSTFLAGS" => flags))
end

n = parse(Int64, ARGS[1])

# Compile-time options: fat LTO, codegen-units=1, target-cpu=native

FAT_LTO = "-C lto=fat -C embed-bitcode=yes"
CODEGEN_UNITS = "-C codegen-units=1"
CPU_NATIVE = "-C target-cpu=native"

samples = DataFrame((time=[], LTO=[], CGU=[], CPU_NATIVE=[]))

for a in ["", FAT_LTO]
    for b in ["", CODEGEN_UNITS]
        for c in ["", CPU_NATIVE]
            settings = (LTO = Int(a != ""),
                CGU = Int(b != ""),
                CPU_NATIVE = Int(c != ""))
            flags = join([a, b, c], " ")
            println("compiling with $flags")
            compile(flags)
            times = bench(n)
            println("mean: ", mean(times[2:end]))
            println("stddev: ", std(times[2:end]))

            # skip first sample
            for t in times[2:end]
                push!(samples, merge((time=t,), settings))
            end
        end
    end
end

println("----")
println(samples)
CSV.write("samples.csv", samples)
