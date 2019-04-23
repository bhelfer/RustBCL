filename=$(cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]")
oshrun -n 3 $filename --nocapture
