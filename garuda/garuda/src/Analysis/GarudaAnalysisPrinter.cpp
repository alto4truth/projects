#include "garuda/Analysis/GarudaAnalysisPrinter.h"
#include "garuda/Analysis/GarudaAnalysis.h"
#include <llvm/IR/Module.h>
#include <llvm/IR/Function.h>
#include <llvm/Support/raw_ostream.h>

namespace garuda {

class AnalysisPrinterPass : public llvm::ModulePass {
public:
    static char ID;

    AnalysisPrinterPass();
    ~AnalysisPrinterPass() override;

    bool runOnModule(llvm::Module& module) override;
    void getAnalysisUsage(llvm::AnalysisUsage& info) const override;

private:
    llvm::raw_ostream* os;
};

char AnalysisPrinterPass::ID = 0;

AnalysisPrinterPass::AnalysisPrinterPass() : llvm::ModulePass(ID), os(nullptr) {}

AnalysisPrinterPass::~AnalysisPrinterPass() = default;

bool AnalysisPrinterPass::runOnModule(llvm::Module& module) {
    if (!os) return false;

    *os << "Garuda Analysis Results\n";
    *os << "====================\n\n";

    *os << "Module: " << module.getName().data() << "\n";
    *os << "Number of functions: " << module.size() << "\n\n";

    for (auto& function : module) {
        *os << "Function: " << function.getName().data() << "\n";
        *os << "  Number of basic blocks: " << function.size() << "\n";
        *os << "  Number of arguments: " << function.arg_size() << "\n";
    }

    return true;
}

void AnalysisPrinterPass::getAnalysisUsage(llvm::AnalysisUsage& info) const {
    info.setPreservesAll();
}

std::unique_ptr<GarudaAnalysisPrinter> createAnalysisPrinter(llvm::raw_ostream& out) {
    return std::make_unique<GarudaAnalysisPrinter>(out);
}

GarudaAnalysisPrinter::GarudaAnalysisPrinter(llvm::raw_ostream& out) : os(&out) {}

GarudaAnalysisPrinter::~GarudaAnalysisPrinter() = default;

void GarudaAnalysisPrinter::printAnalysisResult(llvm::Module& module, const AnalysisResult& result) {
    if (!os) return;

    *os << "Garuda Analysis Results\n";
    *os << "====================\n\n";

    *os << "Module: " << module.getName().data() << "\n";
    *os << "Converged: " << (result.isConverged() ? "yes" : "no") << "\n";
    *os << "Iterations: " << result.getIterationCount() << "\n\n";

    const auto& violations = result.getViolations();
    if (!violations.empty()) {
        *os << "Violations:\n";
        for (const auto& pair : violations) {
            *os << "  " << pair.first << " at " << pair.second << "\n";
        }
        *os << "\n";
    }

    const auto& warnings = result.getWarnings();
    if (!warnings.empty()) {
        *os << "Warnings:\n";
        for (const auto& pair : warnings) {
            *os << "  " << pair.first << " at " << pair.second << "\n";
        }
        *os << "\n";
    }
}

void GarudaAnalysisPrinter::printGroupDomains(const AnalysisResult& result) {
    if (!os) return;

    *os << "Group Domains:\n";
    for (const auto& pair : result.getGroupDomains()) {
        *os << "  " << pair.first << "\n";
    }
    *os << "\n";
}

void GarudaAnalysisPrinter::printProperties(const AnalysisResult& result) {
    if (!os) return;

    *os << "Properties:\n";
    for (const auto& pair : result.getProperties()) {
        *os << "  " << pair.first << " = " << pair.second << "\n";
    }
    *os << "\n";
}

void GarudaAnalysisPrinter::printStatistics(const AnalysisResult& result) {
    if (!os) return;

    *os << "Statistics:\n";
    *os << "  Violations: " << result.getViolations().size() << "\n";
    *os << "  Warnings: " << result.getWarnings().size() << "\n";
    *os << "  Notes: " << result.getNotes().size() << "\n";
    *os << "  Group Domains: " << result.getGroupDomains().size() << "\n";
    *os << "\n";
}

void GarudaAnalysisPrinter::flush() {
    if (os) {
        os->flush();
    }
}

llvm::raw_ostream* GarudaAnalysisPrinter::getStream() const {
    return os;
}

void GarudaAnalysisPrinter::setStream(llvm::raw_ostream& out) {
    os = &out;
}

std::map<std::string, std::string> AnalysisResult::getProperties() const {
    std::map<std::string, std::string> props;
    for (const auto& pair : properties) {
        props[pair.first] = pair.second;
    }
    return props;
}

}