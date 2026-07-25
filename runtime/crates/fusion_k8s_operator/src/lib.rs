use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// FusionApp CRD (Custom Resource Definition)
// ---------------------------------------------------------------------------

/// Top-level CRD schema for a Fusion application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FusionApp {
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: FusionAppSpec,
    pub status: Option<FusionAppStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectMeta {
    pub name: String,
    pub namespace: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FusionAppSpec {
    pub image: String,
    pub replicas: u32,
    pub port: u16,
    pub env: HashMap<String, String>,
    pub resources: ResourceRequirements,
    pub auto_scaling: AutoScalingSpec,
    pub health_check: HealthCheckSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceRequirements {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoScalingSpec {
    pub enabled: bool,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_percentage: u32,
    pub target_memory_percentage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheckSpec {
    pub liveness_path: String,
    pub readiness_path: String,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FusionAppStatus {
    pub phase: AppPhase,
    pub ready_replicas: u32,
    pub conditions: Vec<Condition>,
    pub observed_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppPhase {
    Pending,
    Creating,
    Running,
    Scaling,
    Updating,
    Failed,
    Terminating,
}

impl fmt::Display for AppPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppPhase::Pending => write!(f, "Pending"),
            AppPhase::Creating => write!(f, "Creating"),
            AppPhase::Running => write!(f, "Running"),
            AppPhase::Scaling => write!(f, "Scaling"),
            AppPhase::Updating => write!(f, "Updating"),
            AppPhase::Failed => write!(f, "Failed"),
            AppPhase::Terminating => write!(f, "Terminating"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Condition {
    pub type_: String,
    pub status: String,
    pub message: String,
    pub last_transition_time: String,
}

// ---------------------------------------------------------------------------
// Kubernetes resource abstractions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KubeDeployment {
    pub name: String,
    pub namespace: String,
    pub replicas: u32,
    pub image: String,
    pub labels: HashMap<String, String>,
    pub env: HashMap<String, String>,
    pub port: u16,
    pub resource_requirements: ResourceRequirements,
    pub health_check: HealthCheckSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KubeService {
    pub name: String,
    pub namespace: String,
    pub service_type: ServiceType,
    pub port: u16,
    pub target_port: u16,
    pub selector: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KubeConfigMap {
    pub name: String,
    pub namespace: String,
    pub data: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Reconciliation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileAction {
    CreateDeployment,
    UpdateDeployment,
    CreateService,
    UpdateService,
    CreateConfigMap,
    UpdateConfigMap,
    ScaleDeployment(u32),
    None,
}

/// Result produced by the reconciliation loop.
#[derive(Debug, Clone)]
pub struct ReconcileResult {
    pub actions: Vec<ReconcileAction>,
    pub new_status: FusionAppStatus,
}

/// Core operator that owns the reconciliation loop.
pub struct Operator {
    deployments: HashMap<String, KubeDeployment>,
    services: HashMap<String, KubeService>,
    config_maps: HashMap<String, KubeConfigMap>,
}

impl Operator {
    pub fn new() -> Self {
        Self {
            deployments: HashMap::new(),
            services: HashMap::new(),
            config_maps: HashMap::new(),
        }
    }

    /// Run a single reconciliation pass for the given FusionApp.
    pub fn reconcile(&mut self, app: &FusionApp) -> ReconcileResult {
        let mut actions = Vec::new();
        let key = format!("{}/{}", app.metadata.namespace, app.metadata.name);

        // -- Deployment --
        let desired_deployment = self.build_deployment(app);
        match self.deployments.get(&key) {
            None => {
                actions.push(ReconcileAction::CreateDeployment);
                self.deployments.insert(key.clone(), desired_deployment);
            }
            Some(existing) => {
                if existing != &desired_deployment {
                    actions.push(ReconcileAction::UpdateDeployment);
                    self.deployments.insert(key.clone(), desired_deployment.clone());
                }
            }
        }

        // -- Service --
        let desired_service = self.build_service(app);
        match self.services.get(&key) {
            None => {
                actions.push(ReconcileAction::CreateService);
                self.services.insert(key.clone(), desired_service);
            }
            Some(existing) => {
                if existing != &desired_service {
                    actions.push(ReconcileAction::UpdateService);
                    self.services.insert(key.clone(), desired_service);
                }
            }
        }

        // -- ConfigMap --
        let desired_cm = self.build_configmap(app);
        match self.config_maps.get(&key) {
            None => {
                actions.push(ReconcileAction::CreateConfigMap);
                self.config_maps.insert(key.clone(), desired_cm);
            }
            Some(existing) => {
                if existing != &desired_cm {
                    actions.push(ReconcileAction::UpdateConfigMap);
                    self.config_maps.insert(key.clone(), desired_cm);
                }
            }
        }

        let new_status = FusionAppStatus {
            phase: if actions.is_empty() {
                AppPhase::Running
            } else {
                AppPhase::Updating
            },
            ready_replicas: app.spec.replicas,
            conditions: vec![Condition {
                type_: "Reconciled".into(),
                status: "True".into(),
                message: format!("{} action(s) taken", actions.len()),
                last_transition_time: "now".into(),
            }],
            observed_generation: app.metadata.generation,
        };

        ReconcileResult { actions, new_status }
    }

    fn build_deployment(&self, app: &FusionApp) -> KubeDeployment {
        let mut labels = app.metadata.labels.clone();
        labels.insert("app".into(), app.metadata.name.clone());
        labels.insert("managed-by".into(), "fusion-operator".into());

        KubeDeployment {
            name: app.metadata.name.clone(),
            namespace: app.metadata.namespace.clone(),
            replicas: app.spec.replicas,
            image: app.spec.image.clone(),
            labels,
            env: app.spec.env.clone(),
            port: app.spec.port,
            resource_requirements: app.spec.resources.clone(),
            health_check: app.spec.health_check.clone(),
        }
    }

    fn build_service(&self, app: &FusionApp) -> KubeService {
        let mut selector = HashMap::new();
        selector.insert("app".into(), app.metadata.name.clone());

        KubeService {
            name: format!("{}-svc", app.metadata.name),
            namespace: app.metadata.namespace.clone(),
            service_type: ServiceType::ClusterIP,
            port: app.spec.port,
            target_port: app.spec.port,
            selector,
        }
    }

    fn build_configmap(&self, app: &FusionApp) -> KubeConfigMap {
        let mut data = HashMap::new();
        data.insert("APP_NAME".into(), app.metadata.name.clone());
        data.insert("PORT".into(), app.spec.port.to_string());

        KubeConfigMap {
            name: format!("{}-config", app.metadata.name),
            namespace: app.metadata.namespace.clone(),
            data,
        }
    }

    /// Decide the desired replica count based on auto-scaling policy.
    pub fn compute_desired_replicas(
        spec: &AutoScalingSpec,
        current_replicas: u32,
        current_cpu_pct: u32,
        current_memory_pct: u32,
    ) -> u32 {
        if !spec.enabled {
            return current_replicas;
        }

        let mut desired = current_replicas;

        if current_cpu_pct > spec.target_cpu_percentage
            || current_memory_pct > spec.target_memory_percentage
        {
            desired = current_replicas + 1;
        } else if current_cpu_pct < spec.target_cpu_percentage / 2
            && current_memory_pct < spec.target_memory_percentage / 2
            && current_replicas > spec.min_replicas
        {
            desired = current_replicas - 1;
        }

        desired.clamp(spec.min_replicas, spec.max_replicas)
    }

    pub fn get_deployment(&self, key: &str) -> Option<&KubeDeployment> {
        self.deployments.get(key)
    }

    pub fn get_service(&self, key: &str) -> Option<&KubeService> {
        self.services.get(key)
    }
}

impl Default for Operator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_app() -> FusionApp {
        FusionApp {
            api_version: "fusion.io/v2".into(),
            kind: "FusionApp".into(),
            metadata: ObjectMeta {
                name: "my-app".into(),
                namespace: "default".into(),
                labels: HashMap::new(),
                annotations: HashMap::new(),
                generation: 1,
            },
            spec: FusionAppSpec {
                image: "registry.io/my-app:2.0".into(),
                replicas: 3,
                port: 8080,
                env: HashMap::new(),
                resources: ResourceRequirements {
                    cpu_request: "100m".into(),
                    cpu_limit: "500m".into(),
                    memory_request: "128Mi".into(),
                    memory_limit: "512Mi".into(),
                },
                auto_scaling: AutoScalingSpec {
                    enabled: true,
                    min_replicas: 1,
                    max_replicas: 10,
                    target_cpu_percentage: 70,
                    target_memory_percentage: 80,
                },
                health_check: HealthCheckSpec {
                    liveness_path: "/healthz".into(),
                    readiness_path: "/ready".into(),
                    initial_delay_seconds: 5,
                    period_seconds: 10,
                },
            },
            status: None,
        }
    }

    #[test]
    fn reconcile_creates_resources() {
        let mut op = Operator::new();
        let app = sample_app();
        let result = op.reconcile(&app);

        assert!(result.actions.contains(&ReconcileAction::CreateDeployment));
        assert!(result.actions.contains(&ReconcileAction::CreateService));
        assert!(result.actions.contains(&ReconcileAction::CreateConfigMap));
        assert_eq!(result.actions.len(), 3);
    }

    #[test]
    fn reconcile_noop_when_unchanged() {
        let mut op = Operator::new();
        let app = sample_app();
        op.reconcile(&app);

        let result = op.reconcile(&app);
        assert!(result.actions.is_empty());
    }

    #[test]
    fn reconcile_detects_update() {
        let mut op = Operator::new();
        let mut app = sample_app();
        op.reconcile(&app);

        app.spec.replicas = 5;
        let result = op.reconcile(&app);
        assert!(result.actions.contains(&ReconcileAction::UpdateDeployment));
    }

    #[test]
    fn auto_scaler_scales_up() {
        let spec = AutoScalingSpec {
            enabled: true,
            min_replicas: 1,
            max_replicas: 10,
            target_cpu_percentage: 70,
            target_memory_percentage: 80,
        };
        let desired = Operator::compute_desired_replicas(&spec, 3, 85, 50);
        assert_eq!(desired, 4);
    }

    #[test]
    fn auto_scaler_scales_down() {
        let spec = AutoScalingSpec {
            enabled: true,
            min_replicas: 1,
            max_replicas: 10,
            target_cpu_percentage: 70,
            target_memory_percentage: 80,
        };
        let desired = Operator::compute_desired_replicas(&spec, 5, 20, 20);
        assert_eq!(desired, 4);
    }

    #[test]
    fn auto_scaler_respects_bounds() {
        let spec = AutoScalingSpec {
            enabled: true,
            min_replicas: 2,
            max_replicas: 8,
            target_cpu_percentage: 70,
            target_memory_percentage: 80,
        };
        // Would scale to 9 but clamped to max
        assert_eq!(Operator::compute_desired_replicas(&spec, 8, 99, 99), 8);
        // Would scale to 0 but clamped to min
        assert_eq!(Operator::compute_desired_replicas(&spec, 1, 5, 5), 2);
    }

    #[test]
    fn auto_scaler_disabled_no_change() {
        let spec = AutoScalingSpec {
            enabled: false,
            min_replicas: 1,
            max_replicas: 10,
            target_cpu_percentage: 70,
            target_memory_percentage: 80,
        };
        assert_eq!(Operator::compute_desired_replicas(&spec, 3, 99, 99), 3);
    }

    #[test]
    fn serialization_roundtrip() {
        let app = sample_app();
        let json = serde_json::to_string(&app).unwrap();
        let parsed: FusionApp = serde_json::from_str(&json).unwrap();
        assert_eq!(app, parsed);
    }

    #[test]
    fn deployment_has_correct_labels() {
        let mut op = Operator::new();
        let app = sample_app();
        op.reconcile(&app);

        let key = "default/my-app";
        let dep = op.get_deployment(key).unwrap();
        assert_eq!(dep.labels.get("app").unwrap(), "my-app");
        assert_eq!(dep.labels.get("managed-by").unwrap(), "fusion-operator");
    }

    #[test]
    fn service_cluster_ip_type() {
        let mut op = Operator::new();
        let app = sample_app();
        op.reconcile(&app);

        let key = "default/my-app";
        let svc = op.get_service(key).unwrap();
        assert_eq!(svc.service_type, ServiceType::ClusterIP);
        assert_eq!(svc.port, 8080);
    }
}
