# concon

Simple kube controller that will deploy a compose file.

To test:

* install the CRD `kubectl apply -f controller/yamls/crd.yaml`
* in controller run `cargo run`, wait a bit
* `kubectl apply -f examples/example-hello-world.dockerapp/docker-compose.yml`
* look for the pod: `kubectl get pod`
* port forwad it: `kubectl port-forward pod/.... 5678:5678`
* test it: `curl localhost:5678`
* remove the deployment: `kubectl delete compose.compose.rumpl.dev/echo`

Seven easy-peasy steps.
