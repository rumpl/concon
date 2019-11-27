use compose_yml::v2::{CommandLine, File, Ports::Port, Service};
use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::{
    api::{Api, DeleteParams, Informer, Object, PostParams, RawApi, Void, WatchEvent},
    client::APIClient,
    config,
};
use serde_json;
use std::collections::BTreeMap;

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
        WatchEvent::Deleted(_book) => delete_deployment(),
        _ => println!("another event"),
    }
}

fn delete_deployment() {
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);
    let deployments = Api::v1Deployment(client).within("default");
    println!("Deleting  Deployment");
    let dp = DeleteParams::default();
    match deployments.delete("hello-deployment", &dp) {
        Ok(_o) => println!("Deleted"),
        Err(e) => eprintln!("Unable to delete deployment {}", e),
    }
}

fn create_pod(name: &str, service: &Service) {
    let config = config::load_kube_config().expect("failed to load kubeconfig");
    let client = APIClient::new(config);
    let deployments = Api::v1Deployment(client).within("default");
    println!("Creating Deployment");

    let mut map = BTreeMap::new();
    map.insert(String::from("app"), String::from(name));

    let mut map2 = BTreeMap::new();
    map2.insert(String::from("app"), String::from(name));

    let mut map3 = BTreeMap::new();
    map3.insert(String::from("app"), String::from(name));

    let mut ports = Vec::new();
    for port in &service.ports {
        let port_mapping = port.value().unwrap();
        if let Port(port) = port_mapping.container_ports {
            ports.push(i32::from(port))
        }
    }

    let command = service.command.as_ref().unwrap();
    let cmd = match command {
        CommandLine::ShellCode(d) => vec![d.to_string()],
        CommandLine::Parsed(v) => v.iter().map(|x| x.to_string()).collect(),
    };

    let deployment = apps::Deployment {
        metadata: Some(meta::ObjectMeta {
            name: Some(String::from("hello-deployment")),
            labels: Some(map),
            ..Default::default()
        }),
        spec: Some(apps::DeploymentSpec {
            replicas: Some(1),
            selector: meta::LabelSelector {
                match_labels: Some(map2),
                ..Default::default()
            },
            template: api::PodTemplateSpec {
                metadata: Some(meta::ObjectMeta {
                    name: Some(String::from(name)),
                    labels: Some(map3),
                    ..Default::default()
                }),
                spec: Some(api::PodSpec {
                    containers: vec![api::Container {
                        name: String::from(name),
                        image: Some(service.image.as_ref().unwrap().to_string()),
                        args: Some(cmd),
                        ports: Some(vec![api::ContainerPort {
                            container_port: *ports.first().unwrap(),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let pp = PostParams::default();
    match deployments.create(&pp, serde_json::to_vec(&deployment).unwrap()) {
        Ok(o) => {
            println!("Created {}", o.metadata.name);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
