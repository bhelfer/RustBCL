filename=$(cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]")
oshrun -n 2 $filename --nocapture
