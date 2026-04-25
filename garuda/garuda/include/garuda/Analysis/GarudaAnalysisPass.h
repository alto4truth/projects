#ifndef GARUDA_ANALYSIS_PASS_H
#define GARUDA_ANALYSIS_PASS_H

#include "garuda/Analysis/GarudaAnalysis.h"
#include <llvm/Pass.h>
#include <llvm/PassRegistry.h>

namespace garuda {

class GarudaAnalysisPass : public llvm::ModulePass {
public:
    static char ID;

    GarudaAnalysisPass();
    ~GarudaAnalysisPass() override;

    bool runOnModule(llvm::Module& module) override;
    void getAnalysisUsage(llvm::AnalysisUsage& info) const override;

    void setAnalysisAnalysis(const std::string& name, std::shared_ptr<GarudaAnalysis> analysis);
    std::shared_ptr<GarudaAnalysis> getAnalysis(const std::string& name) const;

    llvm::Module* getModule() const { return module; }

private:
    std::map<std::string, std::shared_ptr<GarudaAnalysis>> analyses;
    llvm::Module* module;
};

class FunctionAnalysisPass : public llvm::FunctionPass {
public:
    static char ID;

    FunctionAnalysisPass();
    ~FunctionAnalysisPass() override;

    bool runOnFunction(llvm::Function& function) override;
    void getAnalysisUsage(llvm::AnalysisUsage& info) const override;

    void setAnalysis(std::shared_ptr<GarudaAnalysis> analysis);
    std::shared_ptr<GarudaAnalysis> getAnalysis() const { return analysis; }

    llvm::Function* getFunction() const { return function; }

private:
    std::shared_ptr<GarudaAnalysis> analysis;
    llvm::Function* function;
};



}
#endif