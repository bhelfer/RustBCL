filename=$(cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]")
oshrun -n 4 $filename --nocapture
