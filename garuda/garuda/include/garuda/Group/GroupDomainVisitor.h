#ifndef GARUDA_GROUP_DOMAIN_VISITOR_H
#define GARUDA_GROUP_DOMAIN_VISITOR_H

#include "garuda/Domain/AbstractDomain.h"
#include <string>
#include <memory>
#include <functional>

namespace garuda {

class GroupDomain;

class GroupDomainVisitor {
public:
    virtual ~GroupDomainVisitor() = default;

    virtual void visit(const GroupDomain& domain) = 0;
    virtual void visitEnter(const GroupDomain& domain) = 0;
    virtual void visitLeave(const GroupDomain& domain) = 0;

    virtual std::unique_ptr<AbstractDomain> visitResult(const GroupDomain& domain) = 0;
};

class GroupDomainPrintVisitor : public GroupDomainVisitor {
public:
    GroupDomainPrintVisitor(llvm::raw_ostream& os);
    void visit(const GroupDomain& domain) override;
    void visitEnter(const GroupDomain& domain) override;
    void visitLeave(const GroupDomain& domain) override;
    std::unique_ptr<AbstractDomain> visitResult(const GroupDomain& domain) override;

private:
    llvm::raw_ostream& os;
    size_t depth;
};

class GroupDomainTransformVisitor : public GroupDomainVisitor {
public:
    using TransformFn = std::function<std::unique_ptr<AbstractDomain>(const AbstractDomain&)>;

    explicit GroupDomainTransformVisitor(TransformFn transform);
    void visit(const GroupDomain& domain) override;
    void visitEnter(const GroupDomain& domain) override;
    void visitLeave(const GroupDomain& domain) override;
    std::unique_ptr<AbstractDomain> visitResult(const GroupDomain& domain) override;

    const std::unique_ptr<GroupDomain>& getResult() const { return result; }

private:
    TransformFn transform_fn;
    std::unique_ptr<GroupDomain> result;
};

class GroupDomainCollectorVisitor : public GroupDomainVisitor {
public:
    using FilterFn = std::function<bool(const std::string& key, const AbstractDomain&)>;

    explicit GroupDomainCollectorVisitor(FilterFn filter = nullptr);
    void visit(const GroupDomain& domain) override;
    void visitEnter(const GroupDomain& domain) override;
    void visitLeave(const GroupDomain& domain) override;
    std::unique_ptr<AbstractDomain> visitResult(const GroupDomain& domain) override;

    std::vector<std::string> getCollectedKeys() const { return collected_keys; }
    void reset() { collected_keys.clear(); }

private:
    FilterFn filter_fn;
    std::vector<std::string> collected_keys;
};

}
#endif