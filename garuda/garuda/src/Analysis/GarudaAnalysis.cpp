#include "garuda/Analysis/GarudaAnalysis.h"
#include "garuda/Group/GroupDomainManager.h"
#include "garuda/Update/ProgressUpdateTracker.h"
#include <llvm/IR/Module.h>
#include <llvm/IR/Function.h>
#include <llvm/Analysis/LoopInfo.h>

namespace garuda {

AnalysisResult::AnalysisResult() : converged(false), iteration_count(0) {}

AnalysisResult::~AnalysisResult() = default;

void AnalysisResult::setGroupDomain(const std::string& name, std::shared_ptr<GroupDomain> domain) {
    group_domains[name] = domain;
}

std::shared_ptr<GroupDomain> AnalysisResult::getGroupDomain(const std::string& name) const {
    auto it = group_domains.find(name);
    if (it != group_domains.end()) {
        return it->second;
    }
    return nullptr;
}

bool AnalysisResult::hasGroupDomain(const std::string& name) const {
    return group_domains.find(name) != group_domains.end();
}

void AnalysisResult::addViolation(const std::string& message, const std::string& location) {
    violations.push_back({message, location});
}

void AnalysisResult::addWarning(const std::string& message, const std::string& location) {
    warnings.push_back({message, location});
}

void AnalysisResult::addNote(const std::string& message, const std::string& location) {
    notes.push_back({message, location});
}

void AnalysisResult::setProperty(const std::string& key, const std::string& value) {
    properties[key] = value;
}

std::optional<std::string> AnalysisResult::getProperty(const std::string& key) const {
    auto it = properties.find(key);
    if (it != properties.end()) {
        return it->second;
    }
    return std::nullopt;
}

void AnalysisResult::clear() {
    group_domains.clear();
    violations.clear();
    warnings.clear();
    notes.clear();
    properties.clear();
    converged = false;
    iteration_count = 0;
}

bool AnalysisResult::hasIssues() const {
    return !violations.empty() || !warnings.empty();
}

GarudaAnalysis::GarudaAnalysis()
    : name(""),
      module(nullptr),
      initialized(false) {
    group_manager = std::make_shared<GroupDomainManager>();
    progress_tracker = std::make_shared<ProgressUpdateTracker>();
}

GarudaAnalysis::GarudaAnalysis(const std::string& n)
    : name(n),
      module(nullptr),
      initialized(false) {
    group_manager = std::make_shared<GroupDomainManager>();
    progress_tracker = std::make_shared<ProgressUpdateTracker>(n);
}

GarudaAnalysis::~GarudaAnalysis() = default;

void GarudaAnalysis::initialize(llvm::Module* mod) {
    if (initialized) return;
    module = mod;
    initializeImpl(mod);
    initialized = true;
}

void GarudaAnalysis::finalize() {
    if (!initialized) return;
    finalizeImpl();
    initialized = false;
}

std::shared_ptr<AnalysisResult> GarudaAnalysis::analyze(llvm::Module* mod) {
    if (!initialized) {
        initialize(mod);
    }
    auto result = std::make_shared<AnalysisResult>();
    return result;
}

void GarudaAnalysis::setName(const std::string& n) {
    name = n;
}

void GarudaAnalysis::setGroupManager(std::shared_ptr<GroupDomainManager> manager) {
    group_manager = manager;
}

void GarudaAnalysis::setProgressTracker(std::shared_ptr<ProgressUpdateTracker> tracker) {
    progress_tracker = tracker;
}

void GarudaAnalysis::getAnalysisUsage(llvm::AnalysisUsage& info) const {
    info.setPreservesAll();
}

void GarudaAnalysis::releaseMemory() {
    if (group_manager) {
        group_manager->reset();
    }
    if (progress_tracker) {
        progress_tracker->reset();
    }
}

void GarudaAnalysis::initializeImpl(llvm::Module* mod) {
    module = mod;
}

void GarudaAnalysis::finalizeImpl() {}

FunctionAnalysis::FunctionAnalysis() : GarudaAnalysis("FunctionAnalysis") {}

FunctionAnalysis::~FunctionAnalysis() = default;

std::shared_ptr<AnalysisResult> FunctionAnalysis::analyze(llvm::Function* function) {
    return analyzeFunction(function);
}

LoopAnalysis::LoopAnalysis() : GarudaAnalysis("LoopAnalysis"), loop_info(nullptr) {}

LoopAnalysis::~LoopAnalysis() = default;

std::shared_ptr<AnalysisResult> LoopAnalysis::analyze(llvm::Function* function) {
    auto result = std::make_shared<AnalysisResult>();
    if (!loop_info) {
        return result;
    }
    return analyzeLoop(nullptr, function);
}

void LoopAnalysis::setLoopInfo(llvm::LoopInfo* li) {
    loop_info = li;
}

std::shared_ptr<AnalysisResult> LoopAnalysis::analyzeLoop(llvm::Loop* loop, llvm::Function* function) {
    auto result = std::make_shared<AnalysisResult>();
    return result;
}

}