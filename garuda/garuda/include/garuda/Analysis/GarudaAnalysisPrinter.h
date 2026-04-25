#ifndef GARUDA_ANALYSIS_PRINTER_H
#define GARUDA_ANALYSIS_PRINTER_H

#include "garuda/Analysis/GarudaAnalysisPass.h"
#include <llvm/IR/PrintPass.h>
#include <llvm/Support/raw_ostream.h>

namespace garuda {

class AnalysisResultPrinter : public llvm::ModulePass {
public:
    static char ID;

    explicit AnalysisResultPrinter(llvm::raw_ostream& os);
    ~AnalysisResultPrinter() override;

    bool runOnModule(llvm::Module& module) override;
    void getAnalysisUsage(llvm::AnalysisUsage& info) const override;

private:
    llvm::raw_ostream& os;
};

class AnalysisResultJSONPrinter : public llvm::ModulePass {
public:
    static char ID;

    explicit AnalysisResultJSONPrinter(llvm::raw_ostream& os);
    ~AnalysisResultJSONPrinter() override;

    bool runOnModule(llvm::Module& module) override;
    void getAnalysisUsage(llvm::AnalysisUsage& info) const override;

private:
    void printGroupDomain(llvm::raw_ostream& os, const std::string& name,
                       const GroupDomain& domain, size_t indent);
    void printDomain(llvm::raw_ostream& os, const AbstractDomain& domain, size_t indent);

    llvm::raw_ostream& os;
    bool first;
};

class ViolationReporter {
public:
    ViolationReporter();
    explicit ViolationReporter(llvm::raw_ostream& os);

    void setOutput(llvm::raw_ostream& os);
    void reportViolation(const std::string& message, const std::string& location,
                      const std::string& severity = "error");

    void reportMissingNullCheck(const llvm::Instruction* inst);
    void reportBufferOverflow(const llvm::Instruction* inst, const std::string& details);
    void reportUseAfterFree(const llvm::Value* value, const std::string& location);
    void reportIntegerOverflow(const llvm::Instruction* inst);
    void reportDivisionByZero(const llvm::Instruction* inst);

    void flush();

private:
    llvm::raw_ostream* os;
    std::vector<std::tuple<std::string, std::string, std::string>> violations;
};

}
#endif