filename=$(cargo bench --no-run --message-format=json | jq -r 'select(.target.kind == ["bench"] and .profile.test == true) | .filenames[]')
cp $filename ./target/release/benchmark