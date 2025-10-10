fluvio cluster status || fluvio cluster start --k8
cd ./smart-modules/assign-record-key/ && smdk build && smdk load
cd -
cd ./smart-modules/construct-post-uri/ && smdk build && smdk load
cd -
kind load docker-image loaded-pg:$POSTGRES_VERSION

echo "Checking if the embeddings-server is running at port 8080"
if ! curl -s -o /dev/null localhost:8080; then
  echo "Starting TEI"
  export model=jinaai/jina-embeddings-v2-base-en
  text-embeddings-router --model-id $model --port 8080 --auto-truncate &

  # Wait until the service is up
  while ! curl -s -o /dev/null localhost:8080; do
    echo "Waiting for TEI to start..."
    sleep 2
  done

  echo "TEI is up!"
fi
