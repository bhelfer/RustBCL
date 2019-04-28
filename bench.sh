filename=$(cargo bench --no-run --message-format=json | jq -r 'select(.target.kind == ["bench"] and .profile.test == true) | .filenames[]')
oshrun -n 3 $filename #--nocapture
