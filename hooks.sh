fluvio cluster status || fluvio cluster start --k8
cd ./smart-modules/assign-record-key/ && smdk build && smdk load
cd -
cd ./smart-modules/contruct-post-uri/ && smdk build && smdk load
cd -
kind load docker-image loaded-pg:$POSTGRES_VERSION
