use compose_yml::v2::{File, Service};
use k8s_openapi::api::apps::v1 as apps;
use kube::{
    api::{Api, Informer, Object, PostParams, RawApi, Void, WatchEvent},
    client::APIClient,
    config,
};
use serde_json::json;

type KubeFile = Object<File, Void>;

fn main() {
    let kubeconfig = config::load_kube_config().expect("kubeconfig failed to load");
    let client = APIClient::new(kubeconfig);
    let namespace = "default";
    let resource = RawApi::customResource("composes")
        .group("compose.rumpl.dev")
        .within(&namespace);
    let informer = Informer::raw(client, resource)
        .init()
        .expect("informer init failed");
    println!("Informer init completed");
    loop {
        informer.poll().expect("informer poll failed");
        while let Some(event) = informer.pop() {
            handle(event);
        }
    }
}

fn handle(event: WatchEvent<KubeFile>) {
    match event {
        WatchEvent::Added(compose) => {
            for (name, service) in &compose.spec.services {
                println!("Added {}", name);
                create_pod(name, &service);
            }
        }
        WatchEvent::Modified(_book) => println!("Modified"),
        WatchEvent::Deleted(_book) => println!("Deleted"),
        _ => println!("another event"),
    }
}

fn create_pod(name: &str, service: &Service) {
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);
    let deployments = Api::v1Deployment(client).within("default");
    println!("Creating Deployment");
    // TODO: use kube openapi for this
    let p = json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": "hello-deployment",
            "labels": {
                "app": name,
            }
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": {
                    "app": name,
                },
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": name
                    },
                },
                "spec": {
                "containers": [{
                        "name": name,
                        "image": service.image.as_ref().unwrap(),
                        "args": ["-text", "hello"],
                        "ports": [{
                            "containerPort": 5678,
                        }],
                    }],
                },
            },
        },
    });

    let pp = PostParams::default();
    match deployments.create(&pp, serde_json::to_vec(&p).unwrap()) {
        Ok(o) => {
            println!("Created {}", o.metadata.name);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
