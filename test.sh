filename=$(cargo test --no-run --message-format=json | jq -r 'select(.profile.test == true and .target.kind == ["bin"]) | .filenames[]')
oshrun -n 3 $filename --nocapture
