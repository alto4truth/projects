#include "garuda/Group/GroupDomainVisitor.h"
#include "garuda/Group/GroupDomain.h"
#include <llvm/Support/raw_ostream.h>

namespace garuda {

void GroupDomainVisitor::visit(const GroupDomain& domain) {}

void GroupDomainVisitor::visitEnter(const GroupDomain& domain) {}

void GroupDomainVisitor::visitLeave(const GroupDomain& domain) {}

std::unique_ptr<AbstractDomain> GroupDomainVisitor::visitResult(const GroupDomain& domain) {
    return domain.clone();
}

GroupDomainPrintVisitor::GroupDomainPrintVisitor(llvm::raw_ostream& out) : os(out), depth(0) {}

void GroupDomainPrintVisitor::visit(const GroupDomain& domain) {
    visitEnter(domain);
    visitLeave(domain);
}

void GroupDomainPrintVisitor::visitEnter(const GroupDomain& domain) {
    for (size_t i = 0; i < depth; ++i) os << "  ";
    os << "Group(" << domain.getName() << "):\n";
    for (size_t i = 0; i < depth + 1; ++i) os << "  ";
    os << "{\n";
    ++depth;

    auto keys = domain.getMemberKeys();
    for (size_t i = 0; i < keys.size(); ++i) {
        for (size_t j = 0; j < depth; ++j) os << "  ";
        auto member = domain.getMember(keys[i]);
        if (member) {
            os << keys[i] << " -> " << member->toString() << "\n";
        }
    }
}

void GroupDomainPrintVisitor::visitLeave(const GroupDomain& domain) {
    --depth;
    for (size_t i = 0; i < depth; ++i) os << "  ";
    os << "}\n";
}

std::unique_ptr<AbstractDomain> GroupDomainPrintVisitor::visitResult(const GroupDomain& domain) {
    return domain.clone();
}

GroupDomainTransformVisitor::GroupDomainTransformVisitor(TransformFn transform)
    : transform_fn(std::move(transform)) {}

void GroupDomainTransformVisitor::visit(const GroupDomain& domain) {
    visitEnter(domain);
    visitLeave(domain);
}

void GroupDomainTransformVisitor::visitEnter(const GroupDomain& domain) {}

void GroupDomainTransformVisitor::visitLeave(const GroupDomain& domain) {
    result = std::make_unique<GroupDomain>(domain.getName(), domain.getConfig());
    auto keys = domain.getMemberKeys();
    for (const auto& key : keys) {
        auto member = domain.getMember(key);
        if (member) {
            auto transformed = transform_fn(*member);
            result->addMember(key, std::move(transformed));
        }
    }
    auto deps = domain.getDependencies("");
    for (const auto& dep : deps) {
        result->addDependency("", dep);
    }
}

std::unique_ptr<AbstractDomain> GroupDomainTransformVisitor::visitResult(const GroupDomain& domain) {
    if (result) {
        return std::move(result);
    }
    return domain.clone();
}

GroupDomainCollectorVisitor::GroupDomainCollectorVisitor(FilterFn filter)
    : filter_fn(std::move(filter)) {}

void GroupDomainCollectorVisitor::visit(const GroupDomain& domain) {
    visitEnter(domain);
    visitLeave(domain);
}

void GroupDomainCollectorVisitor::visitEnter(const GroupDomain& domain) {
    auto keys = domain.getMemberKeys();
    for (const auto& key : keys) {
        auto member = domain.getMember(key);
        if (member) {
            if (!filter_fn || filter_fn(key, *member)) {
                collected_keys.push_back(key);
            }
        }
    }
}

void GroupDomainCollectorVisitor::visitLeave(const GroupDomain& domain) {}

std::unique_ptr<AbstractDomain> GroupDomainCollectorVisitor::visitResult(const GroupDomain& domain) {
    return domain.clone();
}

}