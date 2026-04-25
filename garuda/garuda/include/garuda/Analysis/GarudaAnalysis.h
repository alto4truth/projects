#ifndef GARUDA_ANALYSIS_H
#define GARUDA_ANALYSIS_H

#include "garuda/Group/GroupDomainManager.h"
#include "garuda/Update/ProgressUpdateTracker.h"
#include <llvm/IR/Module.h>
#include <llvm/IR/Function.h>
#include <llvm/IR/Instructions.h>
#include <llvm/Analysis/LoopInfo.h>
#include <memory>
#include <map>
#include <string>
#include <vector>

namespace garuda {

class AnalysisResult {
public:
    AnalysisResult();
    ~AnalysisResult();

    void setGroupDomain(const std::string& name, std::shared_ptr<GroupDomain> domain);
    std::shared_ptr<GroupDomain> getGroupDomain(const std::string& name) const;
    bool hasGroupDomain(const std::string& name) const;

    void addViolation(const std::string& message, const std::string& location);
    const std::vector<std::pair<std::string, std::string>>& getViolations() const { return violations; }

    void addWarning(const std::string& message, const std::string& location);
    const std::vector<std::pair<std::string, std::string>>& getWarnings() const { return warnings; }

    void addNote(const std::string& message, const std::string& location);
    const std::vector<std::pair<std::string, std::string>>& getNotes() const { return notes; }

    void setConverged(bool value) { converged = value; }
    bool isConverged() const { return converged; }

    void setIterationCount(size_t count) { iteration_count = count; }
    size_t getIterationCount() const { return iteration_count; }

    void setProperty(const std::string& key, const std::string& value);
    std::optional<std::string> getProperty(const std::string& key) const;
    std::map<std::string, std::string> getProperties() const;

    void clear();
    bool hasIssues() const;

private:
    std::map<std::string, std::shared_ptr<GroupDomain>> group_domains;
    std::vector<std::pair<std::string, std::string>> violations;
    std::vector<std::pair<std::string, std::string>> warnings;
    std::vector<std::pair<std::string, std::string>> notes;
    std::map<std::string, std::string> properties;
    bool converged;
    size_t iteration_count;
};

class GarudaAnalysis {
public:
    GarudaAnalysis();
    explicit GarudaAnalysis(const std::string& name);
    virtual ~GarudaAnalysis();

    virtual void initialize(llvm::Module* module);
    virtual void finalize();

    virtual std::shared_ptr<AnalysisResult> analyze(llvm::Function* function) = 0;
    virtual std::shared_ptr<AnalysisResult> analyze(llvm::Module* module);

    void setName(const std::string& name);
    const std::string& getName() const { return name; }

    void setGroupManager(std::shared_ptr<GroupDomainManager> manager);
    std::shared_ptr<GroupDomainManager> getGroupManager() const { return group_manager; }

    void setProgressTracker(std::shared_ptr<ProgressUpdateTracker> tracker);
    std::shared_ptr<ProgressUpdateTracker> getProgressTracker() const { return progress_tracker; }

    virtual bool isAnalysisPass() const { return true; }
    virtual llvm::StringRef getPassName() const { return name; }

    virtual void getAnalysisUsage(llvm::AnalysisUsage& info) const;
    virtual void releaseMemory();

protected:
    virtual void initializeImpl(llvm::Module* module);
    virtual void finalizeImpl();

    std::string name;
    llvm::Module* module;
    std::shared_ptr<GroupDomainManager> group_manager;
    std::shared_ptr<ProgressUpdateTracker> progress_tracker;
    bool initialized;
};

class FunctionAnalysis : public GarudaAnalysis {
public:
    FunctionAnalysis();
    ~FunctionAnalysis() override;

    std::shared_ptr<AnalysisResult> analyze(llvm::Function* function) override;

protected:
    virtual std::shared_ptr<AnalysisResult> analyzeFunction(llvm::Function* function) = 0;
};

class LoopAnalysis : public GarudaAnalysis {
public:
    LoopAnalysis();
    ~LoopAnalysis() override;

    std::shared_ptr<AnalysisResult> analyze(llvm::Function* function) override;
    void setLoopInfo(llvm::LoopInfo* li);
    llvm::LoopInfo* getLoopInfo() const { return loop_info; }

protected:
    virtual std::shared_ptr<AnalysisResult> analyzeLoop(llvm::Loop* loop, llvm::Function* function);
    llvm::LoopInfo* loop_info;
};

}
#endif