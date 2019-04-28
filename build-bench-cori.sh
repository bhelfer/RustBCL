# run 'build-bench-cori.sh' on the shell without interactive session.
# run 'run-bench-cori.sh' on the shell with interactive session.
# check https://bheisler.github.io/criterion.rs/book/criterion_rs.html for how to write benchmark file.

filename=$(cargo bench --no-run --message-format=json | jq -r 'select(.target.kind == ["bench"] and .profile.test == true) | .filenames[]')
cp $filename ./target/release/benchmark