//! Benchmark task definition (Issue 78).
//!
//! A concrete, reproducible decision task used identically across
//! all three conditions: Chambers, disposable VM, constrained microVM.

use serde::{Deserialize, Serialize};

/// The benchmark decision task.
/// Same input data across all conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub task_id: String,
    pub question: String,
    pub premises: Vec<TaskPremise>,
    pub constraints: Vec<TaskConstraint>,
    pub alternatives: Vec<TaskAlternative>,
    pub risks: Vec<TaskRisk>,
    pub upsides: Vec<TaskUpside>,
    pub expected_decision: String,
    pub expected_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPremise {
    pub statement: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraint {
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAlternative {
    pub id: String,
    pub description: String,
    pub pros: String,
    pub cons: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRisk {
    pub description: String,
    pub applies_to: String,
    pub likelihood: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpside {
    pub description: String,
    pub applies_to: String,
    pub magnitude: String,
}

/// The canonical benchmark task: cloud provider selection for HIPAA workloads.
pub fn canonical_task() -> BenchmarkTask {
    BenchmarkTask {
        task_id: "cloud-provider-hipaa-v1".into(),
        question: "Choose between three cloud providers based on cost, latency, and compliance for HIPAA-regulated workloads.".into(),
        premises: vec![
            TaskPremise {
                statement: "Organization processes protected health information (PHI) under HIPAA.".into(),
                source: "compliance team".into(),
            },
            TaskPremise {
                statement: "Current on-premise infrastructure reaches end-of-life in 6 months.".into(),
                source: "infrastructure team".into(),
            },
            TaskPremise {
                statement: "Engineering team has strongest expertise in AWS, moderate in Azure, minimal in GCP.".into(),
                source: "engineering leadership".into(),
            },
            TaskPremise {
                statement: "Three peer organizations in the same sector use AWS for HIPAA workloads.".into(),
                source: "industry survey".into(),
            },
        ],
        constraints: vec![
            TaskConstraint {
                description: "Cloud provider must offer a signed HIPAA Business Associate Agreement (BAA).".into(),
                severity: "hard".into(),
            },
            TaskConstraint {
                description: "Monthly infrastructure spend must not exceed $50,000.".into(),
                severity: "hard".into(),
            },
            TaskConstraint {
                description: "Migration must complete within 6 months.".into(),
                severity: "hard".into(),
            },
            TaskConstraint {
                description: "Provider must have at least 2 data center regions in the US.".into(),
                severity: "soft".into(),
            },
        ],
        alternatives: vec![
            TaskAlternative {
                id: "aws".into(),
                description: "Amazon Web Services with HIPAA BAA".into(),
                pros: "Mature platform, broadest service catalog, strong HIPAA track record, largest community, best team expertise fit.".into(),
                cons: "Higher cost at scale, significant vendor lock-in risk, complex pricing model.".into(),
            },
            TaskAlternative {
                id: "azure".into(),
                description: "Microsoft Azure with HIPAA BAA".into(),
                pros: "Strong enterprise integration, competitive pricing, good compliance tooling, Active Directory native.".into(),
                cons: "Moderate team expertise, some services less mature, occasional reliability incidents.".into(),
            },
            TaskAlternative {
                id: "gcp".into(),
                description: "Google Cloud Platform with HIPAA BAA".into(),
                pros: "Superior data analytics, competitive pricing, strong network infrastructure.".into(),
                cons: "Smallest healthcare ecosystem, lowest team expertise, fewer HIPAA-specific reference architectures.".into(),
            },
        ],
        risks: vec![
            TaskRisk {
                description: "Vendor lock-in increases switching cost 3x after year 2 due to proprietary service adoption.".into(),
                applies_to: "aws".into(),
                likelihood: "high".into(),
                impact: "medium".into(),
            },
            TaskRisk {
                description: "Compliance audit failure during migration due to misconfigured security controls.".into(),
                applies_to: "all".into(),
                likelihood: "medium".into(),
                impact: "high".into(),
            },
            TaskRisk {
                description: "Team ramp-up time exceeds 3 months, delaying migration timeline.".into(),
                applies_to: "gcp".into(),
                likelihood: "high".into(),
                impact: "medium".into(),
            },
        ],
        upsides: vec![
            TaskUpside {
                description: "40% reduction in infrastructure management overhead through managed services.".into(),
                applies_to: "aws".into(),
                magnitude: "high".into(),
            },
            TaskUpside {
                description: "Unified identity management with existing Active Directory infrastructure.".into(),
                applies_to: "azure".into(),
                magnitude: "medium".into(),
            },
        ],
        expected_decision: "Select AWS with HIPAA BAA as the primary cloud provider.".into(),
        expected_rationale: "AWS satisfies all hard constraints (HIPAA BAA, budget, timeline, US regions), has the strongest team expertise fit, and the most established HIPAA reference architectures. Vendor lock-in risk is accepted with a documented 3-year exit strategy.".into(),
    }
}
