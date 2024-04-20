using Statistics

function bench(n)
    samples = []
    for _ in 1:n
        t = @elapsed run(pipeline(`./target/release/sudoku-solver`, stdin="sudokus/hardest.txt"))
        push!(samples, t)
    end
    return samples
end

samples = bench(parse(Int64, ARGS[1]))
println(mean(samples))
println(std(samples))
