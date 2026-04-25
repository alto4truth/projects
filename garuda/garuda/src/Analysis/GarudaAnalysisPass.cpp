#include "garuda/Analysis/GarudaAnalysisPass.h"
#include "garuda/Analysis/GarudaAnalysis.h"
#include <llvm/IR/Module.h>
#include <llvm/IR/Function.h>

namespace garuda {

char GarudaAnalysisPass::ID = 0;

GarudaAnalysisPass::GarudaAnalysisPass()
    : llvm::ModulePass(ID), module(nullptr) {}

GarudaAnalysisPass::~GarudaAnalysisPass() = default;

bool GarudaAnalysisPass::runOnModule(llvm::Module& mod) {
    module = &mod;
    for (auto& pair : analyses) {
        auto& analysis = pair.second;
        analysis->initialize(&mod);
        auto result = analysis->analyze(&mod);
        if (result) {
            analysis->finalize();
        }
    }
    return false;
}

void GarudaAnalysisPass::getAnalysisUsage(llvm::AnalysisUsage& info) const {
    info.setPreservesAll();
}

void GarudaAnalysisPass::setAnalysisAnalysis(const std::string& name, std::shared_ptr<GarudaAnalysis> analysis) {
    analyses[name] = analysis;
}

std::shared_ptr<GarudaAnalysis> GarudaAnalysisPass::getAnalysis(const std::string& name) const {
    auto it = analyses.find(name);
    if (it != analyses.end()) {
        return it->second;
    }
    return nullptr;
}

char FunctionAnalysisPass::ID = 0;

FunctionAnalysisPass::FunctionAnalysisPass()
    : llvm::FunctionPass(ID), function(nullptr) {
    analysis = std::make_shared<FunctionAnalysis>();
}

FunctionAnalysisPass::~FunctionAnalysisPass() = default;

bool FunctionAnalysisPass::runOnFunction(llvm::Function& func) {
    function = &func;
    if (analysis) {
        analysis->initialize(function->getParent());
        auto result = analysis->analyze(&func);
        if (result) {
            analysis->finalize();
        }
    }
    return false;
}

void FunctionAnalysisPass::getAnalysisUsage(llvm::AnalysisUsage& info) const {
    info.setPreservesAll();
}

void FunctionAnalysisPass::setAnalysis(std::shared_ptr<GarudaAnalysis> a) {
    analysis = a;
}

}