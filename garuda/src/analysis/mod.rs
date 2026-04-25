use crate::domains::{DomainFactory, DomainState, DomainType, GroupDomain};

pub struct AnalysisResult {
    group_domains: std::collections::HashMap<String, GroupDomain>,
    violations: Vec<(String, String)>,
    warnings: Vec<(String, String)>,
    notes: Vec<(String, String)>,
    properties: std::collections::HashMap<String, String>,
    converged: bool,
    iteration_count: usize,
}

impl AnalysisResult {
    pub fn new() -> Self {
        Self {
            group_domains: std::collections::HashMap::new(),
            violations: Vec::new(),
            warnings: Vec::new(),
            notes: Vec::new(),
            properties: std::collections::HashMap::new(),
            converged: false,
            iteration_count: 0,
        }
    }

    pub fn set_group_domain(&mut self, name: String, domain: GroupDomain) {
        self.group_domains.insert(name, domain);
    }

    pub fn get_group_domain(&self, name: &str) -> Option<&GroupDomain> {
        self.group_domains.get(name)
    }

    pub fn has_group_domain(&self, name: &str) -> bool {
        self.group_domains.contains_key(name)
    }

    pub fn add_violation(&mut self, message: String, location: String) {
        self.violations.push((message, location));
    }

    pub fn get_violations(&self) -> &[(String, String)] {
        &self.violations
    }

    pub fn add_warning(&mut self, message: String, location: String) {
        self.warnings.push((message, location));
    }

    pub fn get_warnings(&self) -> &[(String, String)] {
        &self.warnings
    }

    pub fn add_note(&mut self, message: String, location: String) {
        self.notes.push((message, location));
    }

    pub fn get_notes(&self) -> &[(String, String)] {
        &self.notes
    }

    pub fn set_converged(&mut self, value: bool) {
        self.converged = value;
    }

    pub fn is_converged(&self) -> bool {
        self.converged
    }

    pub fn set_iteration_count(&mut self, count: usize) {
        self.iteration_count = count;
    }

    pub fn get_iteration_count(&self) -> usize {
        self.iteration_count
    }

    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    pub fn clear(&mut self) {
        self.group_domains.clear();
        self.violations.clear();
        self.warnings.clear();
        self.notes.clear();
        self.properties.clear();
    }

    pub fn has_issues(&self) -> bool {
        !self.violations.is_empty() || !self.warnings.is_empty()
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GarudaAnalysis {
    name: String,
    group_manager: Option<GarudaGroupManager>,
    progress_tracker: Option<ProgressUpdateTracker>,
    converged: bool,
    iteration_count: usize,
}

impl GarudaAnalysis {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            group_manager: None,
            progress_tracker: None,
            converged: false,
            iteration_count: 0,
        }
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            group_manager: None,
            progress_tracker: None,
            converged: false,
            iteration_count: 0,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_group_manager(&mut self, manager: GarudaGroupManager) {
        self.group_manager = Some(manager);
    }

    pub fn get_group_manager(&self) -> Option<&GarudaGroupManager> {
        self.group_manager.as_ref()
    }

    pub fn set_progress_tracker(&mut self, tracker: ProgressUpdateTracker) {
        self.progress_tracker = Some(tracker);
    }

    pub fn get_progress_tracker(&self) -> Option<&ProgressUpdateTracker> {
        self.progress_tracker.as_ref()
    }

    pub fn analyze_function<F>(&mut self, function: F) -> AnalysisResult
    where
        F: Fn(&str) -> AnalysisResult,
    {
        let mut result = function(&self.name);
        self.iteration_count += 1;
        result
    }

    pub fn is_converged(&self) -> bool {
        self.converged
    }

    pub fn set_converged(&mut self, value: bool) {
        self.converged = value;
    }
}

impl Default for GarudaAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GarudaGroupManager {
    groups: std::collections::HashMap<String, GroupDomain>,
    config: GroupDomainConfig,
}

#[derive(Debug, Clone)]
pub struct GroupDomainConfig {
    pub enable_widening: bool,
    pub enable_narrowing: bool,
    pub max_iterations: usize,
    pub widening_delay: usize,
    pub track_dependencies: bool,
    pub enable_invariant_checking: bool,
    pub enable_progress_tracking: bool,
}

impl Default for GroupDomainConfig {
    fn default() -> Self {
        Self {
            enable_widening: true,
            enable_narrowing: true,
            max_iterations: 100,
            widening_delay: 3,
            track_dependencies: true,
            enable_invariant_checking: true,
            enable_progress_tracking: true,
        }
    }
}

impl GarudaGroupManager {
    pub fn new() -> Self {
        Self {
            groups: std::collections::HashMap::new(),
            config: GroupDomainConfig::default(),
        }
    }

    pub fn get_group(&self, name: &str) -> Option<&GroupDomain> {
        self.groups.get(name)
    }

    pub fn add_group(&mut self, name: String, domain: GroupDomain) {
        self.groups.insert(name, domain);
    }

    pub fn remove_group(&mut self, name: &str) -> Option<GroupDomain> {
        self.groups.remove(name)
    }
}

impl Default for GarudaGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProgressUpdateTracker {
    updates: Vec<ProgressUpdate>,
    current_stage: String,
    total_stages: usize,
    current_stage_index: usize,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub stage: String,
    pub message: String,
    pub percentage: f64,
    pub timestamp: u64,
}

impl ProgressUpdateTracker {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
            current_stage: String::new(),
            total_stages: 0,
            current_stage_index: 0,
        }
    }

    pub fn add_update(&mut self, update: ProgressUpdate) {
        self.updates.push(update);
    }

    pub fn get_updates(&self) -> &[ProgressUpdate] {
        &self.updates
    }

    pub fn set_stage(&mut self, stage: String) {
        self.current_stage = stage;
    }

    pub fn get_stage(&self) -> &str {
        &self.current_stage
    }

    pub fn next_stage(&mut self) {
        self.current_stage_index += 1;
    }
}

impl Default for ProgressUpdateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: String,
    pub complexity: usize,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
}

impl FunctionAnalysis {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            parameters: Vec::new(),
            return_type: String::new(),
            complexity: 0,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn add_parameter(mut self, name: String, param_type: String) -> Self {
        self.parameters.push(ParameterInfo { name, param_type });
        self
    }

    pub fn with_return_type(mut self, return_type: String) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn with_complexity(mut self, complexity: usize) -> Self {
        self.complexity = complexity;
        self
    }
}

impl Default for FunctionAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
